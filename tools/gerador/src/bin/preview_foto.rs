//! Ferramenta de TESTE, nao usada na geracao final: pega o overlay ja
//! gerado de uma candidatura (em tools/output/<slug>.png) e uma foto
//! qualquer, monta a mesma composicao que o site faz (foto embaixo, overlay
//! em cima) e aplica uma mascara circular preta (diametro = tamanho da
//! imagem) por cima de tudo, pra dar uma nocao de como fica numa foto de
//! perfil de verdade (a maioria das redes sociais recorta o avatar em
//! circulo). A imagem final do site NAO tem essa mascara — isso e so pra
//! visualizacao.
//!
//! Pra gerar o lote de teste inteiro (todas as candidaturas) use o binario
//! `gerar_lote_teste`, que e bem mais rapido — usa assets/teste_bg.png e
//! assets/teste_fg.png ja prontos em 800x800, sem redimensionar nada em
//! tempo de execucao. Esta ferramenta aqui e so pra testar com uma foto
//! qualquer, avulsa (por isso ainda faz o recorte/redimensionamento).

use std::path::PathBuf;

use image::GenericImageView;
use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, PixmapPaint, Rect, Transform};

const TAMANHO: u32 = 800;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("uso: preview_foto <slug> <caminho_da_foto> [saida.png]");
        eprintln!("   (o overlay precisa ja existir em tools/output/<slug>.png — rode o gerar_lote antes)");
        std::process::exit(1);
    }
    let slug = &args[1];
    let foto_path = &args[2];

    let raiz = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let overlay_path = raiz.join("tools/output").join(format!("{}.png", slug));

    let saida_path = if args.len() > 3 {
        PathBuf::from(&args[3])
    } else {
        raiz.join("tools/output_teste").join(format!("{}.png", slug))
    };
    if let Some(pai) = saida_path.parent() {
        std::fs::create_dir_all(pai).unwrap();
    }

    // ---- foto: abre, cobre o quadrado 800x800 e corta o excesso centralizado ----
    let foto = image::open(foto_path).expect("nao consegui abrir a foto");
    let (largura, altura) = foto.dimensions();
    let escala = (TAMANHO as f32 / largura as f32).max(TAMANHO as f32 / altura as f32);
    let nova_larg = (largura as f32 * escala).round().max(1.0) as u32;
    let nova_alt = (altura as f32 * escala).round().max(1.0) as u32;
    let redimensionada = foto.resize_exact(nova_larg, nova_alt, image::imageops::FilterType::Lanczos3);
    let corte_x = (nova_larg.saturating_sub(TAMANHO)) / 2;
    let corte_y = (nova_alt.saturating_sub(TAMANHO)) / 2;
    let cortada = redimensionada.crop_imm(corte_x, corte_y, TAMANHO, TAMANHO);

    // Reaproveita o decoder PNG do proprio tiny-skia (cuida da pre-multiplicacao
    // de alpha certinho) em vez de mexer com PremultipliedColorU8 na mao.
    let mut png_bytes: Vec<u8> = Vec::new();
    cortada
        .write_to(&mut std::io::Cursor::new(&mut png_bytes), image::ImageFormat::Png)
        .expect("falha ao codificar a foto cortada");
    let mut composicao = Pixmap::decode_png(&png_bytes).expect("falha ao decodificar a foto cortada");

    // ---- overlay do candidato por cima ----
    let overlay = Pixmap::load_png(&overlay_path)
        .unwrap_or_else(|_| panic!("nao achei {} — rode o gerar_lote antes", overlay_path.display()));
    composicao.draw_pixmap(0, 0, overlay.as_ref(), &PixmapPaint::default(), Transform::identity(), None);

    // ---- mascara circular preta: retangulo inteiro MENOS o circulo inscrito ----
    let mut pb = PathBuilder::new();
    pb.push_rect(Rect::from_xywh(0.0, 0.0, TAMANHO as f32, TAMANHO as f32).unwrap());
    pb.push_circle(TAMANHO as f32 / 2.0, TAMANHO as f32 / 2.0, TAMANHO as f32 / 2.0);
    if let Some(caminho) = pb.finish() {
        let mut paint = Paint::default();
        paint.set_color_rgba8(0, 0, 0, 255);
        paint.anti_alias = true;
        composicao.fill_path(&caminho, &paint, FillRule::EvenOdd, Transform::identity(), None);
    }

    composicao.save_png(&saida_path).expect("falha ao salvar png");
    println!("gerado: {}", saida_path.display());
}
