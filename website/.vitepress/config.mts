import { defineConfig } from 'vitepress'
import chapters from '../content/_meta/chapters.json'

const bookParts = [...new Set(chapters.map(c => c.part))]
const book = [
  { text: 'Índice', link: '/learn/' },
  ...bookParts.map(part => ({
    text: part, collapsed: true,
    items: chapters.filter(c => c.part === part).map(c => ({ text: c.title, link: c.route }))
  }))
]
const guides = [
  {text:'Todos os guias',link:'/guides/'},
  {text:'Instalação',link:'/guides/installation'},
  {text:'Projeto completo',link:'/guides/project'},
  {text:'Módulos',link:'/guides/modules'},
  {text:'Testes',link:'/guides/testing'},
  {text:'JavaScript e Wasm',link:'/guides/build-targets'},
  {text:'Embedding',link:'/guides/embedding'},
  {text:'Resolver erros',link:'/guides/troubleshooting'}
]
const reference = [
 {text:'Índice',link:'/reference/'},
 {text:'Sintaxe',link:'/reference/syntax'},
 {text:'CLI',link:'/reference/cli'},
 {text:'Biblioteca padrão',link:'/reference/stdlib'},
 {text:'Suporte e backends',link:'/reference/status'},
 {text:'Diagnósticos',link:'/reference/diagnostics'}
]
const concepts = [
 {text:'Índice',link:'/concepts/'},
 {text:'Filosofia',link:'/concepts/philosophy'},
 {text:'Valores e mutação',link:'/concepts/values'},
 {text:'Falhas e contratos',link:'/concepts/failures'},
 {text:'Módulos',link:'/concepts/modules'},
 {text:'Backends',link:'/concepts/backends'}
]
const engineering = [
 {text:'Engenharia — entrada',link:'/engineering/'},
 {text:'Arquitetura',link:'/engineering/architecture'},
 {text:'Frontend',link:'/engineering/frontend'},
 {text:'IR e bytecode',link:'/engineering/ir-bytecode'},
 {text:'Runtime',link:'/engineering/runtime'},
 {text:'JavaScript e Wasm',link:'/engineering/backends'},
 {text:'Stdlib e pacotes',link:'/engineering/stdlib-packages'},
 {text:'Embedded e C ABI',link:'/engineering/embedding'},
 {text:'Qualidade e testes',link:'/engineering/quality'},
 {text:'Acessibilidade',link:'/engineering/accessibility'},
 {text:'Decisões',link:'/engineering/decisions/'},
 {text:'Conflitos conhecidos',link:'/engineering/decisions/conflicts'},
 {text:'Estudos',link:'/engineering/studies'},
 {text:'Protocolo de progresso',link:'/engineering/progress-protocol'},
 {text:'Auditoria de documentos antigos',link:'/engineering/legacy-audit'},
 {text:'Agentes',link:'/engineering/agents/'},
 {text:'Arquivo histórico',link:'/archive/'}
]
export default defineConfig({
  title: 'Aipo',
  description: 'Aipo: aprenda a linguagem e compreenda sua engenharia.',
  srcDir: 'content', outDir: 'dist', publicDir:'public',
  cleanUrls: true, lastUpdated: true,
  head: [['meta',{name:'theme-color',content:'#0f766e'}],['link',{rel:'icon',href:'/logo.svg',type:'image/svg+xml'}]],
  themeConfig: {
    logo:'/logo.svg', siteTitle:'Aipo',
    search: { provider: 'local' },
    nav: [
      {text:'Aprender',link:'/learn/'},
      {text:'Progresso',link:'/progress/'},
      {text:'Guias',link:'/guides/'},
      {text:'Referência',link:'/reference/'},
      {text:'Conceitos',link:'/concepts/'},
      {text:'Engenharia',link:'/engineering/'}
    ],
    sidebar: {
      '/learn/':book, '/guides/':guides, '/reference/':reference,
      '/concepts/':concepts, '/engineering/':engineering, '/archive/':engineering,
      '/progress/':[
        {text:'Painel de progresso',link:'/progress/'},
        {text:'Protocolo de atualização',link:'/engineering/progress-protocol'},
        {text:'Auditoria da documentação antiga',link:'/engineering/legacy-audit'}
      ]
    },
    socialLinks:[{icon:'github',link:'https://github.com/poppy-team/aipo-lang'}],
    editLink:{pattern:'https://github.com/poppy-team/aipo-lang/edit/main/website/content/:path',text:'Editar esta página'},
    docFooter:{prev:'Anterior',next:'Próximo'},
    outline:{label:'Nesta página',level:[2,3]},
    lastUpdated:{text:'Atualizado'},
    footer:{message:'Fonte de verdade única para cada responsabilidade. Nenhuma promessa sem evidência.',copyright:'Poppy Team — Aipo Language'}
  },
  locales:{
    root:{label:'Português',lang:'pt-BR'},
    en:{label:'English (partial)',lang:'en-US',link:'/en/',title:'Aipo',
      themeConfig:{
        nav:[{text:'Home',link:'/en/'},{text:'Getting started',link:'/en/getting-started'},{text:'Português',link:'/learn/'}],
        sidebar:{'/en/':[{text:'Overview',link:'/en/'},{text:'Getting started',link:'/en/getting-started'}]},
        docFooter:{prev:'Previous',next:'Next'},
        outline:{label:'On this page'}
      }
    }
  }
})
