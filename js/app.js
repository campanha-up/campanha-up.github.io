(function () {
  "use strict";

  // ---------- Elementos da página ----------

  var passo1 = document.getElementById("passo-1");
  var passo2 = document.getElementById("passo-2");
  var passo3 = document.getElementById("passo-3");
  var passo4 = document.getElementById("passo-4");
  var linkWhatsapp = document.getElementById("link-whatsapp");

  var inputFoto = document.getElementById("input-foto");
  var areaRecorte = document.getElementById("area-recorte");
  var fotoUsuario = document.getElementById("foto-usuario");
  var overlayCampanha = document.getElementById("overlay-campanha");

  var zoomSlider = document.getElementById("zoom-slider");
  var zoomMais = document.getElementById("zoom-mais");
  var zoomMenos = document.getElementById("zoom-menos");

  var btnTrocarFoto = document.getElementById("btn-trocar-foto");
  var btnBaixar = document.getElementById("btn-baixar");

  var mensagemErro = document.getElementById("mensagem-erro");
  var mensagemSucesso = document.getElementById("mensagem-sucesso");

  var canvasExportacao = document.getElementById("canvas-exportacao");

  // Tamanho (em pixels) da imagem final exportada. A moldura da campanha
  // deve ser um PNG quadrado transparente com esta mesma proporção 1:1.
  var TAMANHO_EXPORTACAO = 1080;

  // ---------- Estado do ajuste da foto (posição e zoom) ----------

  var larguraNatural = 0;
  var alturaNatural = 0;
  var escalaBase = 0; // escala mínima que faz a foto cobrir o quadrado
  var deslocamentoX = 0; // em pixels da área de recorte
  var deslocamentoY = 0;
  var urlObjetoAtual = null;

  // ---------- Utilidades ----------

  function esconder(elemento) {
    elemento.hidden = true;
  }

  function mostrar(elemento) {
    elemento.hidden = false;
  }

  function mostrarErro(texto) {
    mensagemErro.textContent = texto;
    mostrar(mensagemErro);
  }

  function limparErro() {
    mensagemErro.textContent = "";
    esconder(mensagemErro);
  }

  function valorZoomAtual() {
    return parseFloat(zoomSlider.value);
  }

  // ---------- Link de compartilhar no WhatsApp ----------

  if (linkWhatsapp) {
    var textoCompartilhar =
      "Fiz minha foto de perfil pra Samara Presidente ✊ Bora fazer a sua também?\n" +
      window.location.href;
    linkWhatsapp.href = "https://wa.me/?text=" + encodeURIComponent(textoCompartilhar);
  }

  // ---------- Escolha da foto ----------

  inputFoto.addEventListener("change", function (evento) {
    var arquivo = evento.target.files && evento.target.files[0];
    if (!arquivo) {
      return;
    }

    if (arquivo.type.indexOf("image/") !== 0) {
      mostrarErro("Ops! Escolha um arquivo de imagem (uma foto) pra continuar.");
      return;
    }

    limparErro();

    if (urlObjetoAtual) {
      URL.revokeObjectURL(urlObjetoAtual);
    }
    urlObjetoAtual = URL.createObjectURL(arquivo);

    fotoUsuario.onload = function () {
      larguraNatural = fotoUsuario.naturalWidth;
      alturaNatural = fotoUsuario.naturalHeight;
      // Precisa mostrar a área de recorte ANTES de medi-la, senão
      // clientWidth fica em 0 (elemento ainda escondido) e a foto
      // não aparece.
      mostrar(passo2);
      mostrar(passo3);
      prepararEdicao();
      passo2.scrollIntoView({ behavior: "smooth", block: "start" });
    };

    fotoUsuario.onerror = function () {
      mostrarErro(
        "Ops! Não consegui abrir essa foto. Tente outra imagem (formato JPG ou PNG)."
      );
    };

    fotoUsuario.src = urlObjetoAtual;
  });

  btnTrocarFoto.addEventListener("click", function () {
    inputFoto.value = "";
    inputFoto.click();
  });

  // ---------- Preparação da área de ajuste ----------

  function prepararEdicao() {
    zoomSlider.value = "1";
    deslocamentoX = 0;
    deslocamentoY = 0;
    atualizarEscalaBase();
    aplicarTransformacao();
  }

  function atualizarEscalaBase() {
    var tamanhoArea = areaRecorte.clientWidth;
    if (!tamanhoArea || !larguraNatural || !alturaNatural) {
      return;
    }
    escalaBase = Math.max(tamanhoArea / larguraNatural, tamanhoArea / alturaNatural);
  }

  function escalaAtual() {
    return escalaBase * valorZoomAtual();
  }

  // Garante que a foto sempre cubra totalmente o quadrado de recorte,
  // não deixando o usuário arrastar a foto para fora da moldura.
  function limitarDeslocamento() {
    var tamanhoArea = areaRecorte.clientWidth;
    var escala = escalaAtual();
    var larguraExibida = larguraNatural * escala;
    var alturaExibida = alturaNatural * escala;

    var limiteX = Math.max(0, (larguraExibida - tamanhoArea) / 2);
    var limiteY = Math.max(0, (alturaExibida - tamanhoArea) / 2);

    deslocamentoX = Math.min(limiteX, Math.max(-limiteX, deslocamentoX));
    deslocamentoY = Math.min(limiteY, Math.max(-limiteY, deslocamentoY));
  }

  function aplicarTransformacao() {
    limitarDeslocamento();
    var tamanhoArea = areaRecorte.clientWidth;
    var escala = escalaAtual();
    var larguraExibida = larguraNatural * escala;
    var alturaExibida = alturaNatural * escala;

    var esquerda = (tamanhoArea - larguraExibida) / 2 + deslocamentoX;
    var topo = (tamanhoArea - alturaExibida) / 2 + deslocamentoY;

    fotoUsuario.style.width = larguraExibida + "px";
    fotoUsuario.style.height = alturaExibida + "px";
    fotoUsuario.style.transform = "translate(" + esquerda + "px, " + topo + "px)";
  }

  window.addEventListener("resize", function () {
    if (!larguraNatural) {
      return;
    }
    atualizarEscalaBase();
    aplicarTransformacao();
  });

  // ---------- Arrastar a foto (mouse, toque e caneta) ----------

  var arrastando = false;
  var inicioPonteiroX = 0;
  var inicioPonteiroY = 0;
  var inicioDeslocamentoX = 0;
  var inicioDeslocamentoY = 0;

  // Suporte a pinça (dois dedos) para aproximar/afastar no celular.
  var ponteirosAtivos = new Map();
  var distanciaInicialPinca = 0;
  var zoomInicialPinca = 1;

  function distanciaEntrePonteiros() {
    var pontos = Array.from(ponteirosAtivos.values());
    var dx = pontos[0].x - pontos[1].x;
    var dy = pontos[0].y - pontos[1].y;
    return Math.sqrt(dx * dx + dy * dy);
  }

  areaRecorte.addEventListener("pointerdown", function (evento) {
    if (!larguraNatural) {
      return;
    }
    areaRecorte.setPointerCapture(evento.pointerId);
    ponteirosAtivos.set(evento.pointerId, { x: evento.clientX, y: evento.clientY });

    if (ponteirosAtivos.size === 1) {
      arrastando = true;
      inicioPonteiroX = evento.clientX;
      inicioPonteiroY = evento.clientY;
      inicioDeslocamentoX = deslocamentoX;
      inicioDeslocamentoY = deslocamentoY;
    } else if (ponteirosAtivos.size === 2) {
      arrastando = false;
      distanciaInicialPinca = distanciaEntrePonteiros();
      zoomInicialPinca = valorZoomAtual();
    }
  });

  areaRecorte.addEventListener("pointermove", function (evento) {
    if (!ponteirosAtivos.has(evento.pointerId)) {
      return;
    }
    ponteirosAtivos.set(evento.pointerId, { x: evento.clientX, y: evento.clientY });

    if (ponteirosAtivos.size === 2) {
      var novaDistancia = distanciaEntrePonteiros();
      if (distanciaInicialPinca > 0) {
        var fator = novaDistancia / distanciaInicialPinca;
        definirZoom(zoomInicialPinca * fator);
      }
      return;
    }

    if (!arrastando) {
      return;
    }
    deslocamentoX = inicioDeslocamentoX + (evento.clientX - inicioPonteiroX);
    deslocamentoY = inicioDeslocamentoY + (evento.clientY - inicioPonteiroY);
    aplicarTransformacao();
  });

  function finalizarPonteiro(evento) {
    ponteirosAtivos.delete(evento.pointerId);
    if (ponteirosAtivos.size < 2) {
      distanciaInicialPinca = 0;
    }
    if (ponteirosAtivos.size === 0) {
      arrastando = false;
    }
  }

  areaRecorte.addEventListener("pointerup", finalizarPonteiro);
  areaRecorte.addEventListener("pointercancel", finalizarPonteiro);
  areaRecorte.addEventListener("pointerleave", function (evento) {
    if (ponteirosAtivos.size <= 1) {
      finalizarPonteiro(evento);
    }
  });

  // Roda do mouse para aproximar/afastar no computador.
  areaRecorte.addEventListener(
    "wheel",
    function (evento) {
      if (!larguraNatural) {
        return;
      }
      evento.preventDefault();
      var passo = evento.deltaY < 0 ? 0.08 : -0.08;
      definirZoom(valorZoomAtual() + passo);
    },
    { passive: false }
  );

  // Mover a foto pelo teclado (setas), para quem navega sem mouse/toque.
  areaRecorte.addEventListener("keydown", function (evento) {
    if (!larguraNatural) {
      return;
    }
    var passoTeclado = 20;
    var moveu = true;
    if (evento.key === "ArrowLeft") {
      deslocamentoX += passoTeclado;
    } else if (evento.key === "ArrowRight") {
      deslocamentoX -= passoTeclado;
    } else if (evento.key === "ArrowUp") {
      deslocamentoY += passoTeclado;
    } else if (evento.key === "ArrowDown") {
      deslocamentoY -= passoTeclado;
    } else {
      moveu = false;
    }
    if (moveu) {
      evento.preventDefault();
      aplicarTransformacao();
    }
  });

  // ---------- Controles de zoom (botões e slider) ----------

  function definirZoom(novoValor) {
    var min = parseFloat(zoomSlider.min);
    var max = parseFloat(zoomSlider.max);
    var valorLimitado = Math.min(max, Math.max(min, novoValor));
    zoomSlider.value = String(valorLimitado);
    aplicarTransformacao();
  }

  zoomSlider.addEventListener("input", function () {
    aplicarTransformacao();
  });

  zoomMais.addEventListener("click", function () {
    definirZoom(valorZoomAtual() + 0.15);
  });

  zoomMenos.addEventListener("click", function () {
    definirZoom(valorZoomAtual() - 0.15);
  });

  // ---------- Download da foto final ----------

  btnBaixar.addEventListener("click", function () {
    if (!larguraNatural) {
      return;
    }

    esconder(mensagemSucesso);

    var contexto = canvasExportacao.getContext("2d");
    canvasExportacao.width = TAMANHO_EXPORTACAO;
    canvasExportacao.height = TAMANHO_EXPORTACAO;
    contexto.clearRect(0, 0, TAMANHO_EXPORTACAO, TAMANHO_EXPORTACAO);

    // Recalcula a mesma posição/escala usada na pré-visualização,
    // mas em resolução final (TAMANHO_EXPORTACAO x TAMANHO_EXPORTACAO).
    var tamanhoArea = areaRecorte.clientWidth;
    var fatorExportacao = TAMANHO_EXPORTACAO / tamanhoArea;
    var escala = escalaAtual() * fatorExportacao;

    var larguraExibida = larguraNatural * escala;
    var alturaExibida = alturaNatural * escala;
    var esquerda = (TAMANHO_EXPORTACAO - larguraExibida) / 2 + deslocamentoX * fatorExportacao;
    var topo = (TAMANHO_EXPORTACAO - alturaExibida) / 2 + deslocamentoY * fatorExportacao;

    contexto.drawImage(fotoUsuario, esquerda, topo, larguraExibida, alturaExibida);

    function finalizarDownload() {
      contexto.drawImage(overlayCampanha, 0, 0, TAMANHO_EXPORTACAO, TAMANHO_EXPORTACAO);
      canvasExportacao.toBlob(function (blob) {
        if (!blob) {
          mostrarErro("Não foi possível gerar a imagem. Tente novamente.");
          return;
        }
        var urlDownload = URL.createObjectURL(blob);
        var link = document.createElement("a");
        link.href = urlDownload;
        link.download = "foto-perfil-samara-martins-up80.png";
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
        URL.revokeObjectURL(urlDownload);
        mostrar(mensagemSucesso);
        mostrar(passo4);
      }, "image/png");
    }

    if (overlayCampanha.complete) {
      finalizarDownload();
    } else {
      overlayCampanha.onload = finalizarDownload;
    }
  });
})();
