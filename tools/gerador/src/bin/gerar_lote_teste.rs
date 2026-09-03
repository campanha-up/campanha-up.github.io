//! Gera o lote de TESTE (tools/output_teste) a partir do lote normal
//! (tools/output) ja pronto. Nada de redimensionar foto ou desenhar
//! mascara em tempo de execucao — assets/teste_bg.png e assets/teste_fg.png
//! ja sao 800x800 prontos, entao isso e so 3 composicoes de PNG por
//! candidatura (fundo, overlay, mascara por cima), tudo em paralelo.

use std::fs;
use std::path::PathBuf;

use rayon::prelude::*;
use tiny_skia::{Pixmap, PixmapPaint, Transform};

fn main() {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let raiz = crate_dir.parent().unwrap().parent().unwrap().to_path_buf();

    let saida_dir = raiz.join("tools/output");
    let saida_teste_dir = raiz.join("tools/output_teste");
    fs::create_dir_all(&saida_teste_dir).unwrap();

    let teste_bg = Pixmap::load_png(crate_dir.join("assets/teste_bg.png"))
        .expect("nao achei assets/teste_bg.png");
    let teste_fg = Pixmap::load_png(crate_dir.join("assets/teste_fg.png"))
        .expect("nao achei assets/teste_fg.png");

    let entradas: Vec<_> = fs::read_dir(&saida_dir)
        .unwrap_or_else(|e| panic!("nao consegui listar {}: {e}", saida_dir.display()))
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "png"))
        .collect();

    entradas.par_iter().for_each(|entrada| {
        let overlay = Pixmap::load_png(entrada.path())
            .unwrap_or_else(|e| panic!("nao consegui ler {}: {e}", entrada.path().display()));

        let mut composicao = teste_bg.clone();
        composicao.draw_pixmap(0, 0, overlay.as_ref(), &PixmapPaint::default(), Transform::identity(), None);
        composicao.draw_pixmap(0, 0, teste_fg.as_ref(), &PixmapPaint::default(), Transform::identity(), None);

        let caminho = saida_teste_dir.join(entrada.file_name());
        composicao.save_png(&caminho).expect("falha ao salvar png");
    });

    println!("{} imagens de teste geradas em {}", entradas.len(), saida_teste_dir.display());
}
