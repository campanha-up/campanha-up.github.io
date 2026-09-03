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

  var candidaturaFotoEl = document.getElementById("candidatura-foto");
  var candidaturaNomeEl = document.getElementById("candidatura-nome");
  var candidaturaCargoEl = document.getElementById("candidatura-cargo");
  var btnTrocarCandidatura = document.getElementById("btn-trocar-candidatura");
  var seletorCandidatura = document.getElementById("seletor-candidatura");
  var selectEstado = document.getElementById("select-estado");
  var selectCargo = document.getElementById("select-cargo");
  var selectNome = document.getElementById("select-nome");

  if (candidaturaFotoEl) {
    candidaturaFotoEl.addEventListener("load", function () {
      candidaturaFotoEl.hidden = false;
    });
    candidaturaFotoEl.addEventListener("error", function () {
      candidaturaFotoEl.hidden = true;
    });
  }

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

  // ---------- Candidatura selecionada (estado, cargo, pessoa) ----------

  var CANDIDATURAS = window.CANDIDATURAS || [];
  var ORDEM_CARGOS = window.ORDEM_CARGOS || [];
  var candidaturaAtual = null;

  function candidaturaPorSlug(slug) {
    for (var i = 0; i < CANDIDATURAS.length; i++) {
      if (CANDIDATURAS[i].slug === slug) return CANDIDATURAS[i];
    }
    return null;
  }

  function estadosDisponiveis() {
    var vistos = {};
    var lista = [];
    CANDIDATURAS.forEach(function (c) {
      var chave = c.uf || "";
      if (!vistos[chave]) {
        vistos[chave] = true;
        lista.push({ uf: c.uf, estado: c.estado });
      }
    });
    lista.sort(function (a, b) {
      if (a.uf === null) return -1;
      if (b.uf === null) return 1;
      return a.estado.localeCompare(b.estado, "pt-BR");
    });
    return lista;
  }

  function cargosDisponiveis(uf) {
    var rotulos = {};
    CANDIDATURAS.forEach(function (c) {
      if ((c.uf || "") === (uf || "")) {
        rotulos[c.cargo] = c.cargoRotulo;
      }
    });
    return ORDEM_CARGOS.filter(function (codigo) {
      return rotulos[codigo];
    }).map(function (codigo) {
      return { cargo: codigo, cargoRotulo: rotulos[codigo] };
    });
  }

  function candidatosDisponiveis(uf, cargo) {
    return CANDIDATURAS.filter(function (c) {
      return (c.uf || "") === (uf || "") && c.cargo === cargo;
    }).sort(function (a, b) {
      return a.nome.localeCompare(b.nome, "pt-BR");
    });
  }

  function popularSelectEstado(ufSelecionado) {
    selectEstado.innerHTML = "";
    estadosDisponiveis().forEach(function (e) {
      var opcao = document.createElement("option");
      opcao.value = e.uf || "";
      opcao.textContent = e.uf ? e.uf + " — " + e.estado : "Nacional";
      selectEstado.appendChild(opcao);
    });
    selectEstado.value = ufSelecionado || "";
  }

  function popularSelectCargo(uf, cargoSelecionado) {
    selectCargo.innerHTML = "";
    var cargos = cargosDisponiveis(uf);
    cargos.forEach(function (c) {
      var opcao = document.createElement("option");
      opcao.value = c.cargo;
      opcao.textContent = c.cargoRotulo;
      selectCargo.appendChild(opcao);
    });
    var existe = cargos.some(function (c) {
      return c.cargo === cargoSelecionado;
    });
    selectCargo.value = existe ? cargoSelecionado : cargos.length ? cargos[0].cargo : "";
  }

  function popularSelectNome(uf, cargo, slugSelecionado) {
    selectNome.innerHTML = "";
    var candidatos = candidatosDisponiveis(uf, cargo);
    candidatos.forEach(function (c) {
      var opcao = document.createElement("option");
      opcao.value = c.slug;
      opcao.textContent = c.nome;
      selectNome.appendChild(opcao);
    });
    var existe = candidatos.some(function (c) {
      return c.slug === slugSelecionado;
    });
    selectNome.value = existe ? slugSelecionado : candidatos.length ? candidatos[0].slug : "";
  }

  function atualizarLinkWhatsapp() {
    if (!linkWhatsapp || !candidaturaAtual) {
      return;
    }
    var textoCompartilhar =
      "Fiz minha foto de perfil pra " + candidaturaAtual.nome + " (UP 80) ✊ Bora fazer a sua também?\n" +
      window.location.origin + "/" + candidaturaAtual.slug;
    linkWhatsapp.href = "https://wa.me/?text=" + encodeURIComponent(textoCompartilhar);
  }

  function slugParaUrl(slug) {
    return "/" + slug;
  }

  function sincronizarSelects(candidatura) {
    popularSelectEstado(candidatura.uf || "");
    popularSelectCargo(candidatura.uf, candidatura.cargo);
    popularSelectNome(candidatura.uf, candidatura.cargo, candidatura.slug);
  }

  function aplicarCandidatura(candidatura, opcoes) {
    opcoes = opcoes || {};
    candidaturaAtual = candidatura;

    overlayCampanha.src = "assets/overlays/" + candidatura.slug + ".png";
    overlayCampanha.alt = "Filtro da campanha " + candidatura.nome + " UP 80";

    if (candidaturaFotoEl) {
      candidaturaFotoEl.hidden = true;
      candidaturaFotoEl.alt = "Foto de perfil de " + candidatura.nome;
      candidaturaFotoEl.src = "assets/fotos/" + candidatura.slug + ".jpg";
    }

    candidaturaNomeEl.textContent = candidatura.nome;
    candidaturaCargoEl.textContent = candidatura.uf
      ? candidatura.cargoRotulo + " — " + candidatura.uf
      : candidatura.cargoRotulo;

    document.title = candidatura.nome + " — Filtro de perfil (UP 80)";

    sincronizarSelects(candidatura);
    atualizarLinkWhatsapp();

    if (!opcoes.semAtualizarUrl) {
      var novaUrl = slugParaUrl(candidatura.slug);
      if (window.location.pathname !== novaUrl) {
        window.history.pushState({ slug: candidatura.slug }, "", novaUrl);
      }
    }
  }

  function slugAtualDaUrl() {
    var redirecionado = null;
    try {
      redirecionado = window.sessionStorage.getItem("candidaturaRedirect");
    } catch (erro) {
      redirecionado = null;
    }
    if (redirecionado) {
      try {
        window.sessionStorage.removeItem("candidaturaRedirect");
      } catch (erro) {
        /* ignora navegadores sem sessionStorage */
      }
      window.history.replaceState(null, "", slugParaUrl(redirecionado));
      return redirecionado;
    }
    var caminho = window.location.pathname.replace(/^\/+|\/+$/g, "");
    return caminho || null;
  }

  if (btnTrocarCandidatura && seletorCandidatura) {
    btnTrocarCandidatura.addEventListener("click", function () {
      var estaAberto = !seletorCandidatura.hidden;
      seletorCandidatura.hidden = estaAberto;
      btnTrocarCandidatura.setAttribute("aria-expanded", String(!estaAberto));
      btnTrocarCandidatura.textContent = estaAberto ? "Trocar candidatura" : "Fechar";
    });
  }

  selectEstado.addEventListener("change", function () {
    var uf = selectEstado.value || null;
    popularSelectCargo(uf);
    var candidatos = candidatosDisponiveis(uf, selectCargo.value);
    popularSelectNome(uf, selectCargo.value, candidatos.length ? candidatos[0].slug : "");
    var candidatura = candidaturaPorSlug(selectNome.value);
    if (candidatura) {
      aplicarCandidatura(candidatura);
    }
  });

  selectCargo.addEventListener("change", function () {
    var uf = selectEstado.value || null;
    popularSelectNome(uf, selectCargo.value);
    var candidatura = candidaturaPorSlug(selectNome.value);
    if (candidatura) {
      aplicarCandidatura(candidatura);
    }
  });

  selectNome.addEventListener("change", function () {
    var candidatura = candidaturaPorSlug(selectNome.value);
    if (candidatura) {
      aplicarCandidatura(candidatura);
    }
  });

  window.addEventListener("popstate", function () {
    var caminho = window.location.pathname.replace(/^\/+|\/+$/g, "") || window.CANDIDATURA_PADRAO;
    var candidatura = candidaturaPorSlug(caminho) || candidaturaPorSlug(window.CANDIDATURA_PADRAO);
    aplicarCandidatura(candidatura, { semAtualizarUrl: true });
  });

  (function inicializarCandidatura() {
    var slugInicial = slugAtualDaUrl();
    var candidatura = (slugInicial && candidaturaPorSlug(slugInicial)) || candidaturaPorSlug(window.CANDIDATURA_PADRAO);
    var precisaCorrigirUrl = Boolean(slugInicial) && slugInicial !== candidatura.slug;
    aplicarCandidatura(candidatura, { semAtualizarUrl: !precisaCorrigirUrl });
    if (precisaCorrigirUrl) {
      window.history.replaceState({ slug: candidatura.slug }, "", slugParaUrl(candidatura.slug));
    }
  })();

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
        link.download = "foto-perfil-" + candidaturaAtual.slug + "-up80.png";
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
