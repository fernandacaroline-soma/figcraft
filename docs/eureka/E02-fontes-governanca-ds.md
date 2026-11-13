# E02 — Fontes oficiais e governança do DS

Card Notion E02 · Eureka Design · leitura feita em 13/11/2026 · **proposta para revisão da dona do produto**

Legenda de evidência: **verificado (fonte)** = li hoje na fonte citada. **inferido** = dedução minha, não confirmada. **não acessado** = não consegui ler.

## Resumo

1. A regra visual (tokens, modos, layout, componentes) vive no Figma do **DS N Marcas** e nos arquivos de marca. A regra de processo (quem cria, quem aprova, como registra) vive no **Notion**.
2. Hoje há pelo menos **quatro cópias** de tokens fora do Figma: o `tokens.json` do artifact Animale, o `tokenRegistry.json` do Docusaurus, o `tokens-figma-button.json` do teste de código e um `tokens.nmarcas.json` **extraído do código de Tecnologia**.
3. Nenhuma tem sincronização automática. Todas são fotos com data.
4. Proposta: o Eureka lê **direto do Figma** (variables e componentes publicados), no formato JSON de variables que o FigCraft já importa, e lê regras de processo **direto do Notion**. Nada de cópia mantida à mão.
5. O código de Tecnologia entra só como relatório de compatibilidade. Nunca vira padrão.

## Mapa de autoridade

| Tipo de artefato | Onde vive | Quem aprova | Como versiona | Evidência |
|---|---|---|---|---|
| **Tokens** (cor, espaço, raio, borda, opacidade, tipografia) | Figma DS N Marcas `FsU6PD5PxodVk0IVMWQrLP`, coleção **Tokens WEB** (key `b804c201…0bf0`), 312 variables | Liderança de DS. Regra: "não criar token; alteração oficial exige autorização explícita de governança" | Branch do Figma + publicação da biblioteca; registro no Notion (Camada = Token) | Coleção e key: verificado (Figma, Plugin API). Regra: verificado (Notion, Fluxo Oficial de Criação e Alteração). Total do arquivo: 326 variables (312 + 14 do Layout); 104 com descrição, **0 com codeSyntax**, 10 com alias, 13 ocultas da publicação: verificado (Figma) |
| **Modos de marca** | Mesma coleção Tokens WEB, 12 modos | Liderança de DS | Igual aos tokens | Verificado (Figma). Regra "componente original fica neutro; modo só em teste" verificada (Notion, DS N Marcas — Web) |
| **Layout** (Desktop/Mobile) | Figma DS N Marcas, coleção **Layout** (key `d58b310d…e619`), 14 variables, 2 modos | Liderança de DS | Igual aos tokens | Verificado (Figma) |
| **Estilos de texto** | Figma DS N Marcas: 13 estilos locais (Body, Subtitle, Title). 0 estilos de cor e 0 de efeito | Liderança de DS | Biblioteca | Verificado (Figma) |
| **Componentes base e compositions** | Figma DS N Marcas. Páginas: Foundations, 💎 Core Components (~40 páginas), ✨ Compositions (~35), ✨ Modais (10), Templates (Navegação, Institucionais, Editoriais, Parceiros), Login | Designer cria em branch → pré-aprovação (Gatekeeper, IA) → aprovação humana. Base = impacto alto, pode escalar | Branch do Figma + publicação; Registro de alterações | Páginas: verificado (Figma). Fluxo: verificado (Notion, Playbook de Aprovação). **Total de component sets não contado** (só amostra: Button 2 sets, Product Card 7 itens) |
| **Checkout e Account Web** | Arquivo próprio "DS N Marcas \| Checkout e Account WEB" `yjir8KmA8q3DONZUsKg3uK`. Templates, nested e fluxos no mesmo arquivo (exceção oficial) | Mesmo fluxo de aprovação | Branch + publicação | Verificado (Figma: páginas e 1 coleção). Exceção: verificado (Notion, DS N Marcas — Web) |
| **Team Components** (marca) | Arquivos por marca. Animale Web `lzVIxSC2ExCdvvYysBzRJ6` (sem variables locais; consome Tokens WEB, Layout e a coleção do Checkout). NV Web, FARM Web e 7 TCs de App: links na página Design System | Liderança de DS; Produto e Dev confirmam exclusividade (regra escrita para Apps) | Branch + publicação | Bibliotecas do TC Animale: verificado (Figma, teamLibrary). Demais arquivos: links verificados (Notion), arquivos **não abertos** |
| **Regras de uso e de processo** | Notion: Fluxo Oficial de Criação e Alteração, Playbook de Aprovação, Critérios de Escalonamento, Guia de revisão técnica, páginas Web e Apps | Fernanda conduz; Renata e Nataly revisam (página Design System) | Histórico de edição do Notion; adendos datados dentro das páginas | Verificado (Notion). Critérios de Escalonamento e Guia de revisão: **não relidos hoje** |
| **Business Rules** (textos e termos por marca) | Figma, grupo `Business Rule/*` da Tokens WEB (22 string + 17 boolean). Notion manda "valores por marca ficam no Figma; não manter segunda lista" | Liderança de DS (são tokens) | Igual aos tokens | Verificado (Notion, Fluxo Oficial, seção "Tokens de texto por marca"; keys de exemplo citadas lá) |
| **Configuração de componente como variable** | Figma, grupo `Components/*` da Tokens WEB (41 string, 77 boolean, 5 float) | Liderança de DS | Igual aos tokens | Contagem: informada pelo orquestrador. Quem decide o que entra aqui: **não encontrei regra escrita** |
| **Registro de mudanças** | Notion, base "Registro de alterações no Design System" (`76cdc7f3…`). Campos: Camada, Produto, Impacto, Origem, Responsável, Avaliador, Validado Por, Status UX, Status DEV, Link, Jira, Orientação para desenvolvimento | Responsável + Validado Por (pessoas) | Uma linha por mudança, com subitens | Verificado (Notion, schema da base) |
| **Documentação de componente** | (a) Figma: bloco Header → componente → Uso → Specs, mais `description` do componente; (b) Notion "Documentação de Componentes do DS"; (c) Docusaurus `Soma-UXDesign/design-system-docs` (privado, sem deploy, roda local) | Docusaurus: merge humano de PR após conferência docs × Figma | Git (PR com CI de validação) | Repo privado e sem Pages: verificado (GitHub). CI só valida: verificado (`.github/workflows/docs-quality.yml`). `description` existe no Button e Product Card Vertical; vazia nos nested e no Product Card Horizontal: verificado (Figma, amostra) |
| **Perfis de marca** | GitHub `Soma-UXDesign/gerador-propostas`, `…/references/marcas/_modelo.md` + `brand-registry.yaml`. Só a NV tem perfil (`nv.md`, parcial, dona Nataly, 12/11/2026) | Dona de cada perfil (designer da marca), campo "Dona" no modelo | Git (último commit `e3ece86`, 13/11/2026) | Verificado (GitHub, leitura via API). Não existe no repo do FigCraft |
| **Regras operacionais dos agentes** | `Soma-UXDesign/ds-orchestrator` (G01–G12, `references/sources.json`). Diz ser "subordinado ao Notion" | Fernanda (repo); pré-aprovação da IA não substitui humano | Git, versão 0.2.13 | Verificado (README e `guardrails.md`). Precedência escrita: Notion > Figma > decisões aprovadas > Docusaurus (`handoff-policy.md`) |
| **Código de Tecnologia** | `@soma-monorepo/core-components` no monorepo nmarcas. Local: só amostra do Button (zip sem hash de commit) | Tecnologia. **Não define padrão** | Git do monorepo (não acessado) | Amostra: verificado (`teste-codigo-button/RELATORIO.md`). Tokens CSS do código usam nomes derivados do Figma (`Colors/Surface/Brand/Background` → `--colors-surface-brand-background`); o gerador `offpremium-v1` traz nomenclatura antiga: verificado (relatório e memória do teste) |

### Cópias encontradas (todas fotos, nenhuma sincroniza)

| Cópia | Origem | Data | Evidência |
|---|---|---|---|
| `ds-animale-web-figma/project/tokens.json` (artifact DS Animale Web) | Figma, modo Animale + Layout Desktop | 27/10 | Verificado (`meta.synced`) |
| `docs-ds/src/data/tokenRegistry.json` | Figma Tokens WEB | 23/10, `automaticSync: false` | Verificado (`source`) |
| `teste-codigo-button/tokens-figma-button.json` | Figma, 30 tokens do Button | 27/10 | Verificado |
| `animale-design-experiment/design-system/tokens/tokens.nmarcas.json` | **Código de Tecnologia** (`globals.scss` e tema do Storybook do monorepo) | 26/10 | Verificado (`meta.source = github`). É exatamente o caso proibido: token vindo de Dev |

## Proposta de fonte canônica (para revisão)

Regra geral: **uma fonte por tipo; o Eureka lê na hora (ou guarda cache com data e hash) e nunca edita a fonte.**

| Tipo | Fonte canônica proposta | Como o Eureka leria, sem cópia manual |
|---|---|---|
| Tokens, modos, Business Rules, Components-as-variables | Coleção Tokens WEB publicada no DS N Marcas | Script de leitura no Figma (Plugin API, como fiz hoje) que gera o JSON de variables do Figma (`name`, `resolvedType`, `valuesByMode`, aliases `VARIABLE_ALIAS`, `key`, `scopes`, `description`). O FigCraft importa esse formato com `variable.import`. Guardar `fileKey`, `collection.key`, data e hash. Cache vencido = aviso, não fonte |
| Layout | Coleção Layout do DS N Marcas | Mesmo caminho. Manter os modos Desktop/Mobile separados das marcas |
| Componentes base, compositions, Checkout, TCs | Biblioteca publicada de cada arquivo (DS N Marcas, Checkout e Account WEB, TC da marca) | Ler component sets por key: nome, propriedades reais, variantes, `description`, biblioteca de origem. Instância no Eureka aponta para a key; nunca cópia desenhada |
| Regras de uso e processo | Notion: Fluxo Oficial, Playbook, Critérios de Escalonamento, páginas Web/Apps | Ler pelo conector do Notion na hora da decisão. Guardar URL e `last_edited`. O Eureka não reescreve regra; cita |
| Registro de decisões | Base Registro de alterações (Notion) | Leitura para "já decidimos algo parecido?". Escrita só com confirmação humana |
| Documentação para pessoas e agentes | Docusaurus `design-system-docs`, painel "Para agentes" (`_agente.mdx`) | Clone local, ler no HEAD com SHA-256 por arquivo. Só confiar quando `evidence_status = verified_live`; senão, subir ao Figma |
| Perfis de marca | `gerador-propostas/.../references/marcas/` | Ler do Git. Perfil ausente = trabalhar só com usabilidade e DS (regra já escrita no handoff do Eureka) |
| Código de Tecnologia | Monorepo nmarcas | Só relatório de divergência código × Figma (como o do Button). Nunca alimenta tokens nem regras do Eureka |

## Contrato preliminar (rascunho, não implementar)

Campos mínimos de um **token**:

| Campo | Exemplo | Origem do dado |
|---|---|---|
| `nome_amigavel` | "Fundo da marca" | Descrição da variable no Figma (só 104 de 326 têm); se vazio, `null` |
| `nome_tecnico` | `Colors/Surface/Brand/Background` | Nome da variable |
| `key` | key da variable | Figma |
| `colecao` | `Tokens WEB` + key | Figma |
| `tipo` | COLOR, FLOAT, STRING, BOOLEAN | Figma |
| `grupo` | Colors, Spacing, Business Rule, Components… | Primeiro segmento do nome |
| `valores_por_modo` | `{Animale: …, NV: …}` ou alias | `valuesByMode` |
| `alias_de` | key do token referenciado | Figma |
| `escopos` | `FRAME_FILL`… | Figma |
| `nome_css` | `--colors-surface-brand-background` | **Derivado** (não há codeSyntax no Figma). Marcar como derivado |
| `origem` | `figma:FsU6…/b804…` | Fixo |
| `lido_em` / `hash` | data + SHA-256 | Leitor do Eureka |
| `versao_biblioteca` | versão publicada | **Não sei como ler hoje** (pergunta aberta) |
| `aprovacao` | link do Registro, se houver | Notion; `null` quando não há |

Campos mínimos de um **componente**:

| Campo | Observação |
|---|---|
| `nome_amigavel` | Do `description` ou da documentação; `null` se vazio |
| `nome_tecnico` | Nome exato do set no Figma |
| `key` | Key do component set, verificada |
| `biblioteca` | DS N Marcas, Checkout e Account WEB ou TC da marca |
| `camada` | Core, Composition, Nested, Template, Team Component (mesmas opções do Registro) |
| `plataforma` | Web ou App |
| `marca` | `global` ou a marca do TC |
| `propriedades` | Nome completo do Figma (com `#id`), tipo e padrão |
| `modos_suportados` | Marcas e Layout testados |
| `documentacao` | Link Docusaurus e `evidence_status` |
| `lido_em` / `hash` | Leitor do Eureka |
| `aprovacao` | Linha do Registro de alterações (Responsável, Validado Por, Status UX) |
| `compatibilidade_codigo` | Opcional; só informativo, nunca bloqueia nem define |

## Riscos

1. **Cópias desatualizadas.** Quatro arquivos de tokens com datas de 23 a 27/10. O `tokens.json` do artifact Animale só tem cor, espaço, raio e tipografia de um modo. Se o Eureka ler uma cópia, vai divergir sem avisar.
2. **Token vindo de Dev.** O `tokens.nmarcas.json` do experimento Animale foi extraído do código. Se alguém o usar como base, uma regra de Tecnologia vira padrão. Precisa ser marcado como "só compatibilidade" ou aposentado.
3. **Business Rules e Components como variables.** Textos de negócio e configurações de componente moram na mesma coleção dos tokens visuais. Mudar copy exige o fluxo de token. E o Eureka pode tratar um STRING de configuração como copy (o próprio Notion alerta). O leitor precisa separar por grupo.
4. **Terceira biblioteca de Checkout.** O arquivo "DS N Marcas | Checkout e Account WEB" publica uma coleção chamada "Variable collection" com **uma** variable BOOLEAN ("2 meios de pagamento", escopo ALL_SCOPES, modo único). Nome genérico, fora do padrão, e o TC Animale a consome. Não sei se é intencional.
5. **Sem codeSyntax.** Nenhuma variable tem nome de código no Figma. A ponte Figma → CSS é derivada por cada consumidor (Docusaurus, teste do Button). Cada um pode derivar diferente.
6. **Componente global com nome de marca.** O Product Card global tem o nested `.Nested / content-product-farm-etc`. Pode ser exceção aprovada; não verifiquei.
7. **Documentação em três lugares** (Figma, Notion, Docusaurus). O Docusaurus diz que vai virar a superfície principal, mas só depois de aprovação humana por componente.
8. **Matriz do Gatekeeper ainda é proposta de piloto**, segundo o próprio README do orchestrator.

## Perguntas para a dona do produto

1. Confirma a regra "Figma manda em tokens e componentes; Notion manda em processo; Docusaurus é derivado"? A política do orchestrator já diz isso, mas está marcada como operacional.
2. O que fazemos com as quatro cópias de tokens? Aposentar, ou manter com aviso de "foto de [data]"?
3. O `tokens.nmarcas.json` (vindo do código) pode ser apagado ou marcado como "só compatibilidade"?
4. Quem decide o que entra nos grupos `Business Rule` e `Components`? Existe regra escrita que eu não achei?
5. A coleção "Variable collection" do Checkout é intencional? Deveria ter outro nome ou morar em outro lugar?
6. Vale pedir codeSyntax nas variables do Figma para fechar o nome CSS na fonte? (Seria mudança no DS, então é decisão sua.)
7. Como o Eureka deve ler a versão publicada da biblioteca? Histórico de versões do Figma, Registro do Notion, ou outro marco?
8. Perfis de marca: o Eureka deve ler do `gerador-propostas` ou eles vão migrar para outro repo?
9. Regra de Team Component Web: o Notion fala de exclusividade confirmada por Produto e Dev só para Apps. Vale também para Web (Animale, NV, FARM)?

## O que não acessei ou não fiz

- Inventário completo de component sets do DS N Marcas: não feito. Contei só as páginas e uma amostra (Button, Product Card).
- Arquivos DS Apps, TCs de App, NV Web, FARM Web: não abertos.
- Notion: Critérios de Escalonamento, Guia de revisão técnica, Documentação de Componentes do DS e páginas filhas do OKR (Execução, Referências): não relidos hoje. A página do OKR foi editada pela última vez em 14/08 e diz que a fase "Estruturar a Governança" ainda não tinha começado. O status atual não foi conferido.
- Linhas do Registro de alterações: li só o schema, não as linhas.
- Monorepo de Tecnologia: não acessado; só a amostra local do Button.
- `gh search code` não devolveu nada nem para termos que existem. Achei os perfis listando a árvore dos repos da organização.
- API REST de variables do Figma: não testei. A leitura por Plugin API funcionou.
