use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;
use std::time::Instant;

use rayon::prelude::*;
use serde::Deserialize;
use tiny_skia::{
    FillRule, LineJoin, Paint, Path as SkPath, PathBuilder, Pixmap, PixmapPaint, Stroke,
    StrokeDash, Transform as SkTransform,
};
use ttf_parser::{Face, GlyphId, OutlineBuilder};

// Le um .svg direto do disco (sem PNG intermediario cacheado) e rasteriza
// pro tamanho pedido — assim editar o .svg (assets/overlay_*.svg) e ver o
// efeito na proxima rodada, sem precisar re-exportar nada na mao.
fn carregar_svg_como_pixmap(caminho: &std::path::Path, tamanho: u32) -> Pixmap {
    let dados = fs::read(caminho)
        .unwrap_or_else(|e| panic!("nao consegui ler {}: {e}", caminho.display()));
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_data(&dados, &opt)
        .unwrap_or_else(|e| panic!("svg invalido ({}): {e}", caminho.display()));
    let mut pixmap = Pixmap::new(tamanho, tamanho).unwrap();
    resvg::render(&tree, SkTransform::identity(), &mut pixmap.as_mut());
    pixmap
}

// ---------- Config (lida de config.json em tempo de execucao) ----------
//
// Todo ajuste fino (tamanho de fonte, espessura, posicao) mora em
// tools/gerador/config.json, NAO em constantes compiladas — edita o JSON
// e roda `./target/release/gerar_lote` de novo, sem `cargo build`.

#[derive(Deserialize, Clone, Debug)]
struct Config {
    tamanho: f32,
    centro: (f32, f32),
    raio_arco: f32,
    altura_texto_maxima: f32,
    angulo_maximo_graus: f32,
    espessura_contorno: f32,
    espessura_sombra: f32,
    passos_extrusao_sombra: i32,
    rastrejo_extra: f32,
    margem_seguranca: f32,
    cor_nome: (u8, u8, u8),
    cor_cargo: (u8, u8, u8),
    altura_numero_senador: f32,
    altura_numero_dep_federal: f32,
    altura_numero_dep_estadual: f32,
    y_base_numero_senador: f32,
    y_base_numero_dep_federal: f32,
    y_base_numero_dep_estadual: f32,
    espessura_contorno_numero: f32,
    espessura_negrito_numero: f32,
    rastrejo_numero: f32,
    // Presidente e governador(a) usam o mesmo "UP 80" (numero do partido
    // nas eleicoes majoritarias). Deputados tem numero de urna proprio.
    cargos_com_numero_padrao: Vec<String>,
}

static CFG: OnceLock<Config> = OnceLock::new();

fn cfg() -> &'static Config {
    CFG.get().expect("config.json ainda nao foi carregado")
}

fn carregar_config(caminho: &std::path::Path) -> Config {
    let texto = fs::read_to_string(caminho)
        .unwrap_or_else(|e| panic!("nao consegui ler {}: {e}", caminho.display()));
    serde_json::from_str(&texto)
        .unwrap_or_else(|e| panic!("config.json invalido ({}): {e}", caminho.display()))
}

#[derive(Deserialize, Debug)]
struct Candidatura {
    slug: String,
    #[serde(rename = "nomeArco")]
    nome_arco: String,
    cargo: String,
    genero: String,
    #[serde(rename = "cargoArco")]
    cargo_arco: Option<String>,
    numero: Option<String>,
}

fn cargo_genero(cargo: &str, genero: &str) -> &'static str {
    match (cargo, genero) {
        ("presidente", "m") => "Presidente",
        ("presidente", _) => "Presidenta",
        ("governador", "m") => "Governador",
        ("governador", _) => "Governadora",
        ("senador", "m") => "Senador",
        ("senador", _) => "Senadora",
        ("deputado-federal", "m") | ("deputado-estadual", "m") => "Deputado",
        ("deputado-federal", _) | ("deputado-estadual", _) => "Deputada",
        _ => "",
    }
}

// ---------- Fonte ----------

struct Fonte<'a> {
    face: Face<'a>,
    cap_height: f32,
}

impl<'a> Fonte<'a> {
    fn new(data: &'a [u8]) -> Self {
        // sCapHeight da Woodchuck esta errado (600); medimos o "H" de verdade.
        Self::com_referencia(data, 'H')
    }

    fn com_referencia(data: &'a [u8], char_referencia: char) -> Self {
        let face = Face::parse(data, 0).expect("nao consegui ler a fonte");
        let gid = face
            .glyph_index(char_referencia)
            .expect("sem glifo de referencia");
        let rect = face
            .outline_glyph(gid, &mut NulBuilder)
            .expect("sem outline no glifo de referencia");
        let cap_height = (rect.y_max - rect.y_min) as f32;
        Fonte { face, cap_height }
    }

    fn glifo(&self, ch: char) -> Option<GlyphId> {
        self.face.glyph_index(ch)
    }

    fn largura_avanco(&self, ch: char) -> f32 {
        match self.glifo(ch) {
            Some(gid) => self.face.glyph_hor_advance(gid).unwrap_or(0) as f32,
            None => self.face.units_per_em() as f32 * 0.3,
        }
    }

    fn path_glifo<T: Transformador>(&self, ch: char, xf: &T) -> Option<SkPath> {
        let gid = self.glifo(ch)?;
        let mut builder = ColetorTransformado {
            pb: PathBuilder::new(),
            xf,
            iniciado: false,
        };
        self.face.outline_glyph(gid, &mut builder)?;
        builder.pb.finish()
    }
}

trait Transformador {
    fn aplicar(&self, x: f32, y: f32) -> (f32, f32);
}

struct NulBuilder;
impl OutlineBuilder for NulBuilder {
    fn move_to(&mut self, _x: f32, _y: f32) {}
    fn line_to(&mut self, _x: f32, _y: f32) {}
    fn quad_to(&mut self, _x1: f32, _y1: f32, _x: f32, _y: f32) {}
    fn curve_to(&mut self, _x1: f32, _y1: f32, _x2: f32, _y2: f32, _x: f32, _y: f32) {}
    fn close(&mut self) {}
}

// transform_para_angulo equivalente
struct TransformaGlifo {
    cos_t: f32,
    sin_t: f32,
    escala: f32,
    origem_x: f32,
    origem_y: f32,
}

impl TransformaGlifo {
    fn nova(theta: f32, escala: f32, largura_glifo_px: f32, deslocamento_extra: f32) -> Self {
        let cos_t = theta.cos();
        let sin_t = theta.sin();
        let raio_efetivo = cfg().raio_arco - deslocamento_extra;
        let meia_largura = largura_glifo_px / 2.0;
        let origem_x = cfg().centro.0 + raio_efetivo * sin_t - meia_largura * cos_t;
        let origem_y = cfg().centro.1 - raio_efetivo * cos_t - meia_largura * sin_t;
        TransformaGlifo { cos_t, sin_t, escala, origem_x, origem_y }
    }

}

impl Transformador for TransformaGlifo {
    #[inline]
    fn aplicar(&self, gx: f32, gy: f32) -> (f32, f32) {
        let wx = self.origem_x + gx * self.escala * self.cos_t + gy * self.escala * self.sin_t;
        let wy = self.origem_y + gx * self.escala * self.sin_t - gy * self.escala * self.cos_t;
        (wx, wy)
    }
}

// Posiciona um glifo em linha reta (sem curvar no arco) — usada so pro
// texto de debug (raio/altura/angulo), escrito num canto da imagem.
struct TransformaReta {
    escala: f32,
    dx: f32,
    dy: f32,
}

impl Transformador for TransformaReta {
    #[inline]
    fn aplicar(&self, gx: f32, gy: f32) -> (f32, f32) {
        (self.dx + gx * self.escala, self.dy - gy * self.escala)
    }
}

struct ColetorTransformado<'a, T: Transformador> {
    pb: PathBuilder,
    xf: &'a T,
    iniciado: bool,
}

impl<'a, T: Transformador> OutlineBuilder for ColetorTransformado<'a, T> {
    fn move_to(&mut self, x: f32, y: f32) {
        let (wx, wy) = self.xf.aplicar(x, y);
        self.pb.move_to(wx, wy);
        self.iniciado = true;
    }
    fn line_to(&mut self, x: f32, y: f32) {
        let (wx, wy) = self.xf.aplicar(x, y);
        self.pb.line_to(wx, wy);
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let (wx1, wy1) = self.xf.aplicar(x1, y1);
        let (wx, wy) = self.xf.aplicar(x, y);
        self.pb.quad_to(wx1, wy1, wx, wy);
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let (wx1, wy1) = self.xf.aplicar(x1, y1);
        let (wx2, wy2) = self.xf.aplicar(x2, y2);
        let (wx, wy) = self.xf.aplicar(x, y);
        self.pb.cubic_to(wx1, wy1, wx2, wy2, wx, wy);
    }
    fn close(&mut self) {
        self.pb.close();
    }
}

// ---------- Layout no arco ----------

struct ItemLayout {
    ch: char,
    theta: f32,
    escala: f32,
    largura_glifo: f32,
}

fn layout(fonte: &Fonte, texto: &str, altura_texto: f32) -> Vec<ItemLayout> {
    let escala = altura_texto / fonte.cap_height;
    let larguras: Vec<f32> = texto.chars().map(|c| fonte.largura_avanco(c) * escala).collect();
    let fatias: Vec<f32> = larguras.iter().map(|w| w + cfg().rastrejo_extra * altura_texto).collect();
    let angulo_total: f32 = fatias.iter().sum::<f32>() / cfg().raio_arco;

    let mut itens = Vec::with_capacity(texto.chars().count());
    let mut cum = -angulo_total / 2.0;
    for (ch, (largura_glifo, fatia)) in texto.chars().zip(larguras.iter().zip(fatias.iter())) {
        let theta_centro = cum + (largura_glifo / 2.0) / cfg().raio_arco;
        itens.push(ItemLayout { ch, theta: theta_centro, escala, largura_glifo: *largura_glifo });
        cum += fatia / cfg().raio_arco;
    }
    itens
}

fn cabe(fonte: &Fonte, texto: &str, altura_texto: f32) -> bool {
    let itens = layout(fonte, texto, altura_texto);
    let thetas_abs: Vec<f32> = itens.iter().filter(|i| i.ch != ' ').map(|i| i.theta.abs()).collect();
    if thetas_abs.is_empty() {
        return true;
    }
    let theta_min_abs = thetas_abs.iter().cloned().fold(f32::INFINITY, f32::min);
    let theta_max_abs = thetas_abs.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

    if 2.0 * theta_max_abs > cfg().angulo_maximo_graus.to_radians() {
        return false;
    }

    let raio_necessario = cfg().raio_arco + altura_texto + cfg().espessura_contorno + cfg().margem_seguranca;
    let margem_vertical = cfg().centro.1;
    let margem_horizontal = cfg().centro.0.min(cfg().tamanho - cfg().centro.0);

    let topo_ok = raio_necessario * theta_min_abs.cos() <= margem_vertical;
    let lateral_ok = raio_necessario * theta_max_abs.sin() <= margem_horizontal;
    topo_ok && lateral_ok
}

fn altura_ajustada(fonte: &Fonte, texto: &str) -> f32 {
    if cabe(fonte, texto, cfg().altura_texto_maxima) {
        return cfg().altura_texto_maxima;
    }
    let mut baixo = 0.0f32;
    let mut alto = cfg().altura_texto_maxima;
    for _ in 0..40 {
        let meio = (baixo + alto) / 2.0;
        if cabe(fonte, texto, meio) {
            baixo = meio;
        } else {
            alto = meio;
        }
    }
    baixo
}

// ---------- Montagem + rasterizacao ----------

// Escreve `texto` reto (sem curvar no arco), da esquerda pra direita,
// com o pe em (x, y) — usada so pro rotulo de debug.
fn path_texto_reto(fonte: &Fonte, texto: &str, x: f32, y: f32, altura_px: f32) -> Option<SkPath> {
    let escala = altura_px / fonte.cap_height;
    let mut pb = PathBuilder::new();
    let mut cursor_x = x;
    for ch in texto.chars() {
        if ch != ' ' {
            let xf = TransformaReta { escala, dx: cursor_x, dy: y };
            if let Some(p) = fonte.path_glifo(ch, &xf) {
                pb.push_path(&p);
            }
        }
        cursor_x += fonte.largura_avanco(ch) * escala;
    }
    pb.finish()
}

// `nome_primeiro=true` da "NOME CARGO" (so a presidenta usa isso). O
// padrao pra todo mundo e "CARGO NOME". As cores nao mudam com a ordem:
// nome sempre amarelo (cfg().cor_nome), cargo sempre branco (cfg().cor_cargo).
fn montar_pixmap(fonte: &Fonte, nome: &str, cargo: &str, nome_primeiro: bool, debug: bool) -> Pixmap {
    let primeiro_bloco_e_nome = nome_primeiro || cargo.is_empty();

    let texto = if cargo.is_empty() {
        nome.to_uppercase()
    } else if nome_primeiro {
        format!("{}   {}", nome.to_uppercase(), cargo.to_uppercase())
    } else {
        format!("{}   {}", cargo.to_uppercase(), nome.to_uppercase())
    };
    // tamanho (em chars) do bloco que vem primeiro no texto acima.
    let tamanho_primeiro_bloco = if primeiro_bloco_e_nome {
        nome.chars().count()
    } else {
        cargo.chars().count()
    };

    let altura_texto = altura_ajustada(fonte, &texto);
    let itens = layout(fonte, &texto, altura_texto);

    let mut pb_sombra = PathBuilder::new();
    let mut pb_nome = PathBuilder::new();
    let mut pb_cargo = PathBuilder::new();

    for (indice, item) in itens.iter().enumerate() {
        if item.ch == ' ' {
            continue;
        }
        let no_primeiro_bloco = indice < tamanho_primeiro_bloco;
        let eh_nome = no_primeiro_bloco == primeiro_bloco_e_nome;
        let xf_normal = TransformaGlifo::nova(item.theta, item.escala, item.largura_glifo, 0.0);
        if let Some(p) = fonte.path_glifo(item.ch, &xf_normal) {
            if eh_nome {
                pb_nome.push_path(&p);
            } else {
                pb_cargo.push_path(&p);
            }
        }

        for passo in 1..=cfg().passos_extrusao_sombra {
            let deslocamento = cfg().espessura_sombra * passo as f32 / cfg().passos_extrusao_sombra as f32;
            let xf_sombra = TransformaGlifo::nova(item.theta, item.escala, item.largura_glifo, deslocamento);
            if let Some(p) = fonte.path_glifo(item.ch, &xf_sombra) {
                pb_sombra.push_path(&p);
            }
        }
    }

    let mut pixmap = Pixmap::new(cfg().tamanho as u32, cfg().tamanho as u32).unwrap();

    let stroke = Stroke {
        width: cfg().espessura_contorno * 2.0,
        line_join: LineJoin::Miter,
        miter_limit: 3.0,
        ..Default::default()
    };

    // sombra (preta, fill+stroke, atras de tudo)
    if let Some(caminho) = pb_sombra.finish() {
        let mut paint = Paint::default();
        paint.set_color_rgba8(0, 0, 0, 255);
        paint.anti_alias = true;
        pixmap.stroke_path(&caminho, &paint, &stroke, SkTransform::identity(), None);
        pixmap.fill_path(&caminho, &paint, FillRule::Winding, SkTransform::identity(), None);
    }

    // nome (amarelo) e cargo (branco): stroke preto atras, fill na cor por cima
    for (caminho_builder, cor) in [(pb_nome, cfg().cor_nome), (pb_cargo, cfg().cor_cargo)] {
        if let Some(caminho) = caminho_builder.finish() {
            let mut paint_contorno = Paint::default();
            paint_contorno.set_color_rgba8(0, 0, 0, 255);
            paint_contorno.anti_alias = true;
            pixmap.stroke_path(&caminho, &paint_contorno, &stroke, SkTransform::identity(), None);

            let mut paint_fill = Paint::default();
            paint_fill.set_color_rgba8(cor.0, cor.1, cor.2, 255);
            paint_fill.anti_alias = true;
            pixmap.fill_path(&caminho, &paint_fill, FillRule::Winding, SkTransform::identity(), None);
        }
    }

    if debug {
        desenhar_debug(fonte, &mut pixmap, &itens, altura_texto);
    }

    pixmap
}

// Overlay de debug: circulo do raio do arco, cruz no centro, uma linha
// pontilhada do centro ate a ancora de cada letra, e um rotulo com os
// numeros — pra visualizar a geometria por tras do layout.
fn desenhar_debug(fonte: &Fonte, pixmap: &mut Pixmap, itens: &[ItemLayout], altura_texto: f32) {
    let thetas_abs: Vec<f32> = itens.iter().filter(|i| i.ch != ' ').map(|i| i.theta.abs()).collect();
    let theta_max_abs = thetas_abs.iter().cloned().fold(0.0f32, f32::max);
    let angulo_total_deg = 2.0 * theta_max_abs.to_degrees();

    let vermelho = (0xff, 0x00, 0x55);
    let verde = (0x00, 0xcc, 0x44);
    let azul = (0x00, 0xaa, 0xff);

    // circulo tracejado no raio do arco + ponto central
    let mut pb = PathBuilder::new();
    pb.push_circle(cfg().centro.0, cfg().centro.1, cfg().raio_arco);
    if let Some(caminho) = pb.finish() {
        let mut paint = Paint::default();
        paint.set_color_rgba8(vermelho.0, vermelho.1, vermelho.2, 255);
        paint.anti_alias = true;
        let stroke = Stroke {
            width: 2.0,
            dash: StrokeDash::new(vec![8.0, 6.0], 0.0),
            ..Default::default()
        };
        pixmap.stroke_path(&caminho, &paint, &stroke, SkTransform::identity(), None);
    }
    let mut pb = PathBuilder::new();
    pb.push_circle(cfg().centro.0, cfg().centro.1, 4.0);
    if let Some(caminho) = pb.finish() {
        let mut paint = Paint::default();
        paint.set_color_rgba8(vermelho.0, vermelho.1, vermelho.2, 255);
        paint.anti_alias = true;
        pixmap.fill_path(&caminho, &paint, FillRule::Winding, SkTransform::identity(), None);
    }

    // cruz verde marcando o centro
    let mut pb = PathBuilder::new();
    pb.move_to(cfg().centro.0, 0.0);
    pb.line_to(cfg().centro.0, cfg().tamanho);
    pb.move_to(0.0, cfg().centro.1);
    pb.line_to(cfg().tamanho, cfg().centro.1);
    if let Some(caminho) = pb.finish() {
        let mut paint = Paint::default();
        paint.set_color_rgba8(verde.0, verde.1, verde.2, 255);
        paint.anti_alias = true;
        let stroke = Stroke { width: 1.0, ..Default::default() };
        pixmap.stroke_path(&caminho, &paint, &stroke, SkTransform::identity(), None);
    }

    // linha pontilhada + ponto do centro ate cada letra
    let mut pb_linhas = PathBuilder::new();
    let mut pb_pontos = PathBuilder::new();
    for item in itens.iter().filter(|i| i.ch != ' ') {
        let ax = cfg().centro.0 + cfg().raio_arco * item.theta.sin();
        let ay = cfg().centro.1 - cfg().raio_arco * item.theta.cos();
        pb_linhas.move_to(cfg().centro.0, cfg().centro.1);
        pb_linhas.line_to(ax, ay);
        pb_pontos.push_circle(ax, ay, 4.0);
    }
    if let Some(caminho) = pb_linhas.finish() {
        let mut paint = Paint::default();
        paint.set_color_rgba8(azul.0, azul.1, azul.2, 255);
        paint.anti_alias = true;
        let stroke = Stroke {
            width: 1.5,
            dash: StrokeDash::new(vec![4.0, 3.0], 0.0),
            ..Default::default()
        };
        pixmap.stroke_path(&caminho, &paint, &stroke, SkTransform::identity(), None);
    }
    if let Some(caminho) = pb_pontos.finish() {
        let mut paint = Paint::default();
        paint.set_color_rgba8(azul.0, azul.1, azul.2, 255);
        paint.anti_alias = true;
        pixmap.fill_path(&caminho, &paint, FillRule::Winding, SkTransform::identity(), None);
    }

    // rotulo com os numeros, escrito reto no canto inferior esquerdo
    let rotulo = format!(
        "raio={:.0}px altura={:.1}px (max {:.0}px) arco_total={:.1}graus",
        cfg().raio_arco, altura_texto, cfg().altura_texto_maxima, angulo_total_deg
    );
    if let Some(caminho) = path_texto_reto(fonte, &rotulo, 16.0, cfg().tamanho - 16.0, 18.0) {
        let mut paint = Paint::default();
        paint.set_color_rgba8(vermelho.0, vermelho.1, vermelho.2, 255);
        paint.anti_alias = true;
        pixmap.fill_path(&caminho, &paint, FillRule::Winding, SkTransform::identity(), None);
    }
}

// ---------- Numero de urna (senador/deputado) ----------
//
// Tamanho da fonte, posicao vertical, espessura do contorno e do negrito
// falso — tudo isso mora em tools/gerador/config.json agora (chaves
// altura_numero_*, y_base_numero_*, espessura_contorno_numero,
// espessura_negrito_numero). Edita o JSON e roda de novo, sem recompilar.
//
// Um ALTURA/Y_BASE por cargo porque dentro de cada cargo a quantidade de
// digitos e sempre a mesma (regra do TSE: senador 3, dep. federal 4, dep.
// estadual 5) — numeros com mais digitos geralmente pedem fonte menor pra
// nao estourar a largura.

fn altura_numero_por_cargo(cargo: &str) -> f32 {
    match cargo {
        "senador" => cfg().altura_numero_senador,
        "deputado-federal" => cfg().altura_numero_dep_federal,
        "deputado-estadual" => cfg().altura_numero_dep_estadual,
        _ => 0.0,
    }
}

fn y_base_numero_por_cargo(cargo: &str) -> f32 {
    match cargo {
        "senador" => cfg().y_base_numero_senador,
        "deputado-federal" => cfg().y_base_numero_dep_federal,
        "deputado-estadual" => cfg().y_base_numero_dep_estadual,
        _ => cfg().centro.1,
    }
}

// rastrejo_numero e px de espaco extra entre digitos, na escala do
// tamanho da fonte (mesma ideia do rastrejo_extra do arco) — nao entra
// depois do ultimo digito, so ENTRE eles.
fn largura_numero(fonte: &Fonte, digitos: &str, altura: f32) -> f32 {
    let escala = altura / fonte.cap_height;
    let n = digitos.chars().count();
    let largura_glifos: f32 = digitos.chars().map(|c| fonte.largura_avanco(c) * escala).sum();
    let rastrejo_total = cfg().rastrejo_numero * altura * n.saturating_sub(1) as f32;
    largura_glifos + rastrejo_total
}

// So um AVISO (nao corrige nada sozinho) se o valor manual configurado
// fizer o numero passar da borda do circulo — o retangulo do numero
// (largura x altura, base em y_base) tem que caber inteiro dentro do
// raio efetivo (cfg().raio_arco menos a metade do contorno).
fn avisar_se_vazar_circulo(cargo: &str, digitos: &str, largura_total: f32, altura: f32, y_base: f32) {
    let r_efetivo = cfg().raio_arco - cfg().espessura_contorno_numero;
    let cantos = [
        (cfg().centro.0 - largura_total / 2.0, y_base),
        (cfg().centro.0 + largura_total / 2.0, y_base),
        (cfg().centro.0 - largura_total / 2.0, y_base - altura),
        (cfg().centro.0 + largura_total / 2.0, y_base - altura),
    ];
    for (x, y) in cantos {
        let dx = x - cfg().centro.0;
        let dy = y - cfg().centro.1;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist > r_efetivo {
            eprintln!(
                "AVISO: numero \"{digitos}\" ({cargo}) passa da borda do circulo com ALTURA={altura:.1} Y_BASE={y_base:.1} largura_total={largura_total:.1} dist={dist:.1} r_efetivo={r_efetivo:.1} canto=({x:.1},{y:.1}) — ajuste essas chaves em config.json"
            );
            return;
        }
    }
}

// Monta o path do numero (centralizado em cfg().centro.0) e desenha (sombra
// preta grossa atras, preenchimento amarelo — igual ao NOME do arco — por
// cima) direto no pixmap.
fn desenhar_numero(fonte: &Fonte, pixmap: &mut Pixmap, digitos: &str, cargo: &str) {
    let altura = altura_numero_por_cargo(cargo);
    let y_base = y_base_numero_por_cargo(cargo);
    let largura_total = largura_numero(fonte, digitos, altura);
    avisar_se_vazar_circulo(cargo, digitos, largura_total, altura, y_base);
    let escala = altura / fonte.cap_height;

    let rastrejo = cfg().rastrejo_numero * altura;
    let mut pb = PathBuilder::new();
    let mut cursor_x = cfg().centro.0 - largura_total / 2.0;
    for (i, ch) in digitos.chars().enumerate() {
        if i > 0 {
            cursor_x += rastrejo;
        }
        let xf = TransformaReta { escala, dx: cursor_x, dy: y_base };
        if let Some(p) = fonte.path_glifo(ch, &xf) {
            pb.push_path(&p);
        }
        cursor_x += fonte.largura_avanco(ch) * escala;
    }
    let Some(caminho) = pb.finish() else { return };

    // 1) contorno preto — largura cobre o negrito E a espessura do
    // contorno, centrada no traco original do glifo.
    let stroke_contorno = Stroke {
        width: (cfg().espessura_negrito_numero + cfg().espessura_contorno_numero) * 2.0,
        line_join: LineJoin::Miter,
        miter_limit: 3.0,
        ..Default::default()
    };
    let mut paint_contorno = Paint::default();
    paint_contorno.set_color_rgba8(0, 0, 0, 255);
    paint_contorno.anti_alias = true;
    pixmap.stroke_path(&caminho, &paint_contorno, &stroke_contorno, SkTransform::identity(), None);

    // 2) negrito falso: stroke AMARELO por cima, só ate a espessura do
    // negrito — engorda o traco do digito e cobre a parte de dentro do
    // contorno preto, sobrando so a faixa preta externa (cfg().espessura_contorno_numero).
    if cfg().espessura_negrito_numero > 0.0 {
        let stroke_negrito = Stroke {
            width: cfg().espessura_negrito_numero * 2.0,
            line_join: LineJoin::Miter,
            miter_limit: 3.0,
            ..Default::default()
        };
        let mut paint_negrito = Paint::default();
        paint_negrito.set_color_rgba8(cfg().cor_nome.0, cfg().cor_nome.1, cfg().cor_nome.2, 255);
        paint_negrito.anti_alias = true;
        pixmap.stroke_path(&caminho, &paint_negrito, &stroke_negrito, SkTransform::identity(), None);
    }

    // 3) preenchimento por cima, fecha o miolo do traco engordado.
    let mut paint_fill = Paint::default();
    paint_fill.set_color_rgba8(cfg().cor_nome.0, cfg().cor_nome.1, cfg().cor_nome.2, 255);
    paint_fill.anti_alias = true;
    pixmap.fill_path(&caminho, &paint_fill, FillRule::Winding, SkTransform::identity(), None);
}

fn digitos_por_cargo(cargo: &str) -> Option<usize> {
    match cargo {
        "senador" => Some(3),
        "deputado-federal" => Some(4),
        "deputado-estadual" => Some(5),
        _ => None,
    }
}

// ---------- Lote ----------

fn carregar_candidaturas(raiz: &std::path::Path) -> Vec<Candidatura> {
    let saida = Command::new("node")
        .arg("-e")
        .arg(
            "global.window = {}; require('./js/candidaturas.js'); \
             console.log(JSON.stringify(global.window.CANDIDATURAS));",
        )
        .current_dir(raiz)
        .output()
        .expect("falha ao rodar node");
    if !saida.status.success() {
        panic!("node falhou: {}", String::from_utf8_lossy(&saida.stderr));
    }
    serde_json::from_slice(&saida.stdout).expect("json invalido")
}

fn main() {
    let config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config.json");
    CFG.set(carregar_config(&config_path)).unwrap();

    let raiz = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
    let tamanho_px = cfg().tamanho as u32;

    let fonte_path = assets_dir.join("fontes/Woodchuck-Heavy.otf");
    let dados_fonte = fs::read(&fonte_path).expect("nao achei a fonte");

    // Numero do partido (presidente/governador) — lido direto do .svg.
    let numero_padrao = carregar_svg_como_pixmap(&assets_dir.join("overlay_up_80.svg"), tamanho_px);

    // Selo por cima do numero individual — um .svg proprio por cargo
    // (senador/dep. federal/dep. estadual), pra poder ajustar cada um
    // separado (posicao, tamanho) sem afetar os outros dois.
    let overlay_por_cargo: std::collections::HashMap<&str, Pixmap> = [
        ("senador", "overlay_up_senador.svg"),
        ("deputado-federal", "overlay_up_dep_federal.svg"),
        ("deputado-estadual", "overlay_up_dep_estadual.svg"),
    ]
    .into_iter()
    .map(|(cargo, arquivo)| (cargo, carregar_svg_como_pixmap(&assets_dir.join(arquivo), tamanho_px)))
    .collect();

    let fonte_numero_path = assets_dir.join("fontes/etruscan-condensed.otf");
    let dados_fonte_numero = fs::read(&fonte_numero_path).expect("nao achei a etruscan-condensed");

    let saida_dir = raiz.join("tools/output");
    fs::create_dir_all(&saida_dir).unwrap();

    let debug = std::env::args().any(|a| a == "--debug");

    let candidaturas = carregar_candidaturas(&raiz);
    println!(
        "{} candidaturas carregadas{}",
        candidaturas.len(),
        if debug { " (com debug)" } else { "" }
    );

    // Confere que todo mundo que devia ter numero real (de
    // js/candidaturas.js) tem.
    let sem_numero: Vec<&str> = candidaturas
        .iter()
        .filter(|c| digitos_por_cargo(&c.cargo).is_some() && c.numero.is_none())
        .map(|c| c.slug.as_str())
        .collect();
    if !sem_numero.is_empty() {
        println!("AVISO: sem numero real, vou pular o numero desses: {:?}", sem_numero);
    }

    let inicio = Instant::now();

    candidaturas.par_iter().for_each(|cand| {
        let numero = &cand.numero;
        let fonte = Fonte::new(&dados_fonte);
        let cargo_arco = cand
                .cargo_arco
                .clone()
                .unwrap_or_else(|| cargo_genero(&cand.cargo, &cand.genero).to_string());
            let nome_primeiro = cand.cargo == "presidente";
            let mut pixmap = montar_pixmap(&fonte, &cand.nome_arco, &cargo_arco, nome_primeiro, debug);

            if cfg().cargos_com_numero_padrao.iter().any(|c| c == &cand.cargo) {
                pixmap.draw_pixmap(
                    0,
                    0,
                    numero_padrao.as_ref(),
                    &PixmapPaint::default(),
                    SkTransform::identity(),
                    None,
                );
            } else if digitos_por_cargo(&cand.cargo).is_some() {
                if let Some(digitos) = numero {
                    let fonte_numero = Fonte::com_referencia(&dados_fonte_numero, '8');
                    desenhar_numero(&fonte_numero, &mut pixmap, digitos, &cand.cargo);
                }
                if let Some(overlay) = overlay_por_cargo.get(cand.cargo.as_str()) {
                    pixmap.draw_pixmap(
                        0,
                        0,
                        overlay.as_ref(),
                        &PixmapPaint::default(),
                        SkTransform::identity(),
                        None,
                    );
                }
            }

            let caminho = saida_dir.join(format!("{}.png", cand.slug));
            pixmap.save_png(&caminho).expect("falha ao salvar png");
        });

    let duracao = inicio.elapsed();
    println!(
        "{} imagens geradas em {} em {:.3}s",
        candidaturas.len(),
        saida_dir.display(),
        duracao.as_secs_f64()
    );
}
