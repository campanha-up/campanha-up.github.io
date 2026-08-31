# Filtro de perfil — UP 80

Site estático e leve para aplicar o filtro de uma candidatura da Unidade
Popular (UP 80) sobre uma foto de perfil — a de Samara Martins (Presidente)
por padrão, mas com seletor para qualquer uma das candidaturas listadas em
[unidadepopular.org.br/eleicoes](https://unidadepopular.org.br/eleicoes).
Sem bibliotecas externas e sem enviar a foto para nenhum servidor — tudo
acontece no navegador da pessoa.

Projeto independente, feito por um apoiador do partido. Sem vínculo oficial
com a campanha.

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

1. Escolher a candidatura (Samara Martins já vem selecionada).
2. Escolher uma foto.
3. Arrastar e ajustar o zoom até ficar como quiser.
4. Baixar a imagem final.

## Estrutura

```
index.html                     Página
css/style.css                   Estilo
js/app.js                       Seleção de candidatura, escolher foto, arrastar/zoom e baixar
js/candidaturas.js              Lista de candidaturas (slug, nome, cargo, estado)
assets/overlays/<slug>.png      Filtro de cada candidatura, uma por slug
assets/logo-campanha.png        Logo exibida no topo (chapa Samara/Raquel — presidencial)
assets/favicon.ico              Ícone da aba do navegador
assets/fontes/                   Fonte Cooper Hewitt (licença OFL)
404.html                        Redireciona /<slug> pro index.html (ver seção de URLs abaixo)
package.json                    Só para o servidor de desenvolvimento (não afeta o site publicado)
```

## Adicionar ou trocar uma candidatura

Cada candidatura tem um filtro em `assets/overlays/<slug>.png` — um PNG
quadrado (proporção 1:1, 1080×1080 é um bom tamanho) com fundo transparente
onde a foto deve aparecer. O `<slug>` é o mesmo usado em
`unidadepopular.org.br/eleicoes/<slug>`.

- **Trocar o filtro de uma candidatura já cadastrada:** substitua o PNG
  correspondente em `assets/overlays/`, mantendo o nome do arquivo. Todas
  as candidaturas exceto a da Samara ainda estão com um filtro
  provisório gerado automaticamente (círculo tracejado + nome/cargo), só
  pra marcar o lugar até a arte de verdade ficar pronta.
- **Adicionar uma nova candidatura:** acrescente uma entrada em
  `js/candidaturas.js` (slug, nome, cargo, uf, estado) e coloque o PNG
  correspondente em `assets/overlays/<slug>.png`.

## URLs de cada candidatura

Cada candidatura tem sua própria URL para compartilhar, ex:
`https://campanha-up.github.io/samara-martins`. Como o GitHub Pages não
tem servidor de verdade, isso funciona através do `404.html`: quando o
GitHub Pages não encontra aquele caminho, ele serve o `404.html`, que
guarda o slug pedido e redireciona pra página principal, que por sua vez
lê esse valor e mostra a candidatura certa. O redirecionamento é
transparente (a URL final continua sendo `/<slug>`), mas envolve um
recarregamento extra da página.

## Publicar no GitHub Pages

1. Suba os arquivos desta pasta para um repositório no GitHub.
2. Em **Settings → Pages**, escolha **Deploy from a branch**, branch
   `main`, pasta `/ (root)`.
3. O site fica disponível em
   `https://<seu-usuario>.github.io/<nome-do-repositorio>/`.

Não precisa de build — é só HTML, CSS e JavaScript puros.
