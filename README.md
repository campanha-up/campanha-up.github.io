# Filtro de perfil — Vote 13 no 2º turno

Site estático e leve para aplicar o filtro da campanha da Unidade Popular
pelo voto 13 (Lula) no segundo turno sobre uma foto de perfil — contra a
eleição de Flávio Bolsonaro. **Bolsonaro nunca mais! Vamos derrotar o
fascismo!** Sem bibliotecas externas e sem enviar a foto para nenhum
servidor — tudo acontece no navegador da pessoa.

Projeto independente, feito por um apoiador do partido. Sem vínculo oficial
com a campanha.

A versão anterior do site, com um filtro por candidatura da UP 80 no
primeiro turno (e o gerador de filtros em Rust), está guardada na branch
`primeiro-turno-candidaturas-up80`.

## Desenvolvimento

Requer apenas [Node.js](https://nodejs.org/) instalado (Node já vem com o
`npm`). Não há build nem transpilação — o servidor local só serve os
arquivos estáticos com live-reload ao salvar.

```
npm install
npm start
```

Isso abre o site em `http://localhost:5500` e recarrega a página
automaticamente a cada alteração salva.

## Como usar

1. Escolher uma foto.
2. Arrastar e ajustar o zoom até ficar como quiser.
3. Baixar a imagem final.

## Estrutura

```
index.html                Página
css/style.css             Estilo
js/app.js                 Escolher foto, arrastar/zoom e baixar
assets/overlay.png        Filtro da campanha
assets/logo-campanha.png  Banner exibido no topo
assets/favicon.ico        Ícone da aba do navegador
assets/fontes/            Fonte Cooper Hewitt (licença OFL)
package.json              Só para o servidor de desenvolvimento (não afeta o site publicado)
```

## Trocar o filtro

O filtro é `assets/overlay.png` — um PNG quadrado (proporção 1:1) com fundo
transparente onde a foto deve aparecer. Para trocar, basta substituir o
arquivo mantendo o nome. A imagem baixada sai sempre em 1080×1080.

## Publicar no GitHub Pages

1. Suba os arquivos desta pasta para um repositório no GitHub.
2. Em **Settings → Pages**, escolha **Deploy from a branch**, a branch
   publicada e a pasta `/ (root)`.
3. O site fica disponível em
   `https://<seu-usuario>.github.io/<nome-do-repositorio>/`.

Não precisa de build — é só HTML, CSS e JavaScript puros.
