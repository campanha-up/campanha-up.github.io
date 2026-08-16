# Moldura de perfil — Samara UP 80

Site estático e leve para aplicar a moldura da campanha de Samara Martins
(UP 80) sobre uma foto de perfil. Sem bibliotecas externas e sem enviar a
foto para nenhum servidor — tudo acontece no navegador da pessoa.

Projeto independente, feito por um apoiador do partido. Sem vínculo oficial
com a campanha.

## Como usar

1. Escolher uma foto.
2. Arrastar e ajustar o zoom até ficar como quiser.
3. Baixar a imagem final.

## Estrutura

```
index.html                 Página
css/style.css               Estilo
js/app.js                   Escolher foto, arrastar/zoom e baixar
assets/overlay.png          Moldura aplicada sobre a foto
assets/logo-campanha.png    Logo exibida no topo
assets/favicon.ico          Ícone da aba do navegador
assets/fontes/               Fonte Cooper Hewitt (licença OFL)
```

## Trocar a moldura

Substitua `assets/overlay.png` por um PNG quadrado (proporção 1:1) com
fundo transparente onde a foto deve aparecer — 1080×1080 é um bom
tamanho. Mantenha o mesmo nome de arquivo, sem precisar mexer no código.

## Publicar no GitHub Pages

1. Suba os arquivos desta pasta para um repositório no GitHub.
2. Em **Settings → Pages**, escolha **Deploy from a branch**, branch
   `main`, pasta `/ (root)`.
3. O site fica disponível em
   `https://<seu-usuario>.github.io/<nome-do-repositorio>/`.

Não precisa de build — é só HTML, CSS e JavaScript puros.
