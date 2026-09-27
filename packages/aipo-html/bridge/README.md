# aipo.html — Ponte Host JavaScript / DOM

Este diretório contém os scripts de ponte para execução no ambiente host do navegador web:

- **[`aipo-dom.js`](./aipo-dom.js)**: Implementação da API FFI de manipulação do DOM real (`__aipo_dom_*`). Mantém a tabela de ponteiros/handles de nós DOM, delegação centralizada de eventos (`click`, `input`, `change`, `keydown`) e operações cirúrgicas de mutação pontual (`update_text`, `set_attr`, `append_child`).
