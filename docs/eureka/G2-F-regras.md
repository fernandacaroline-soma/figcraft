# G2 · Frente F — Regras do lint de DS

Leitura do Notion feita em 23/11/2026, **só leitura**. Precedência: Notion > Figma > decisões aprovadas (E05 §7).
Severidade: **sempre "aviso"** nesta fase. O lint nunca bloqueia nem corrige.

Páginas lidas:
- **Fluxo Oficial** = [Fluxo Oficial de Criação e Alteração no Design System](https://app.notion.com/p/3a4690e9a7e281e5baefc847c98ee64a) (editada em 23/10/2026)
- **Playbook** = [Playbook | Fluxo Oficial de Aprovação](https://app.notion.com/p/39e690e9a7e2816ca310d7e9f4e1eabb) (20/10/2026)
- **Web** = [DS N Marcas — Web](https://app.notion.com/p/39e690e9a7e281978c0dfc4333a87e8c) (23/07/2026)
- **Escalonamento** = [Critérios de Escalonamento](https://app.notion.com/p/39f690e9a7e281c0bfd9dd5e7d006510) (20/10/2026)
- **E05** = `docs/eureka/E05-arquitetura-adrs.md`, seção 7 (decisões delegadas de 17/11 e 23/11)

## Tabela

| # | Regra (`rule`) | Fonte (link + trecho curto) | O que o lint checa no documento | Severidade | Entra agora? |
|---|---|---|---|---|---|
| 1 | `valor-solto`: valor fixo onde existe token (cor de preenchimento/contorno, tamanho de fonte, gap, padding, raio) | **Fluxo Oficial** §4: "use tokens e variáveis das coleções oficiais". **Escalonamento**, Não aceitar: "exige detach, hardcode ou manutenção manual como regra" | Para cada camada: propriedade com valor e **sem vínculo** de variable (reusa `ds_warnings`), e só quando o documento tem ao menos uma variable que serve àquela propriedade (tipo e escopo). Sem token no documento, não há "onde existe token" | aviso | **Sim** |
| 2 | `escopo`: variable usada fora do escopo declarado | **E05 D4** (decisão de 17/11): "aviso, não erro, até a liderança de DS decidir". Escopos vêm do Figma (`scopes` de cada variable) | (a) Vínculo cuja variable tem `scopes` que não cobre o campo (ex.: Font/Size em raio). (b) D4 ao pé da letra: **padding** vinculado a variable com escopo `GAP` (os `Spacing/*`) | aviso | **Sim** |
| 3 | `marca-nao-escolhida`: tela sem modo de marca explícito | **E05 D3** (23/11): "O modo padrão da coleção no Figma é Farm: a marca deve ser escolhida na tela (`variable.setMode`), nunca deixada ao padrão". **Web** §1 (Telas e Fluxos): "conferir modos de marca" | Frame de topo da página (tela, não componente) que usa variable da coleção de marca (padrão `Tokens WEB`) e não tem modo explícito dela nele nem acima | aviso | **Sim** |
| 4 | `marca-no-componente`: modo de marca preso no componente original | **Web** §3: "nunca deixando um modo específico preso ao componente original". **Fluxo Oficial** §10: "nenhum modo específico ficou preso ao componente original". **Playbook** §3: "Nenhum modo ou valor específico de marca ficou preso à origem" | Componente principal ou conjunto de variantes (e camadas dentro deles) com modo explícito da coleção de marca. Os testes devem ser feitos em instâncias, então qualquer modo no original é aviso (não há "modo de teste" a detectar) | aviso | **Sim** |
| 5 | Variable de coleção não oficial | **Fluxo Oficial** §10: "tokens e variáveis pertencem às coleções corretas". **Playbook** §3: "Tokens e variáveis vêm das coleções corretas" | Vínculo a variable cuja coleção não está na lista oficial | aviso | **Sim (24/11)**: `colecao-nao-oficial`; lista oficial = Tokens APP (App) ou Tokens WEB + Layout (Web), ou `officialCollections` |
| 6 | Override de token em instância de componente oficial | **Fluxo Oficial**, "Tokens — bloqueio crítico": "não trocar nem sobrescrever token de qualquer instância de componente oficial". **Escalonamento** (20/10): "alterar, substituir ou sobrescrever token de componente oficial … não é [permitido]" | Camada dentro de instância cujo vínculo difere do da camada correspondente no componente principal (`source`) | aviso (no Notion é bloqueio) | **Não**: no FigCraft não dá para saber se o componente é "oficial" (pergunta 3) |
| 7 | Componente local no arquivo de Telas e Fluxos | **Web** §1: "Não crie componentes locais no arquivo de Telas e Fluxos" | Componente principal no documento | aviso | **Não**: o documento não diz se é o arquivo de Telas e Fluxos (pergunta 4) |
| 8 | Texto literal onde há token de texto por marca | **Fluxo Oficial** §6, Tokens de texto por marca: "Não trocar por texto literal, romper vínculo" | Texto sem vínculo `characters` | aviso | **Não**: o próprio Notion proíbe escolher token "só pelo nome/igualdade de valor", e checar sem isso acusaria todo texto (pergunta 5) |
| 9 | Referência antiga ou de outra marca | **Playbook** §3: "Não existem referências antigas ou de outra marca" | — | — | **Não**: sem critério mecânico escrito (pergunta 6) |
| 10 | Detach ou reconstrução manual de componente | **Fluxo Oficial** §4: "não faça detach; não reconstrua componentes oficiais". **Playbook**, Bloqueios | — | — | **Não**: o documento não guarda que houve detach; "reconstrução" exige julgamento |

## Como cada achado sai

`ds.lint` devolve `{findings: [{id, name, rule, message, source, prop?, value?}], counted}`. `source` é o link da linha acima (Notion quando há; senão a decisão do E05). `message` em pt-BR, no tom do `summary` das propostas, por exemplo: "Preço: tamanho da fonte 18 está fixo; use um token Font/Size."

## Perguntas para a dona (sem fonte escrita clara)

1. **D4 × Figma.** No Figma (e no FigCraft) o escopo `GAP` cobre gap **e** padding. A D4 trata padding com `Spacing/*` (escopo GAP) como fora do escopo. Mantemos o aviso ou a D4 parte de uma premissa errada?
2. **Coleções oficiais.** Quais coleções contam como "corretas"? Só `Tokens WEB` e `Layout` do DS N Marcas? A "Variable collection" do Checkout entra?
3. **Componente oficial.** Como o Eureka distingue componente oficial de componente local? Pela biblioteca de origem (DS N Marcas, Checkout e Account WEB, TC da marca)?
4. **Tipo de arquivo.** Como o lint sabe que o documento é "Telas e Fluxos" (onde componente local é proibido) e não Checkout e Account (onde é permitido)?
5. **Texto literal.** Existe lista de textos que **devem** ter token Business Rule (ex.: CTA da PDP, sacola vazia)? Sem ela, a regra 8 não tem como ser mecânica.
6. **Outra marca.** O que é "referência antiga ou de outra marca" de forma checável: um token com nome de marca, um modo de outra marca numa tela, outra coisa?
7. **Coleção de marca.** A coleção de marca é sempre `Tokens WEB`? Em Apps a coleção é outra; o lint aceita o nome por parâmetro (`brandCollection`).

## Respostas da dona (24/11)

A dona aprovou as recomendações da sessão principal e definiu que o Eureka começa pelo **DS Apps**.

1. **D4 retirada.** No Figma o escopo `GAP` cobre gap e padding; o aviso de "Spacing em padding" saiu do `ds.lint` e a D4 foi retirada no E05.
2. **Coleções oficiais.**
   - **App (alvo principal):** a coleção **Tokens APP** do DS Apps (`U3pOCU4CEBgYKHGVR1F3sK`).
   - **Web:** Tokens WEB, Layout e a coleção do Checkout e Account.

   Com a lista escrita, a regra 5 pode entrar numa próxima rodada.
3. **Componente oficial:** pela biblioteca de origem (DS Apps, Checkout WL, TC da marca). A regra 6 entra quando o documento guardar a biblioteca de cada instância.
4. **Tipo de arquivo:** o próprio arquivo declara se é "Telas e Fluxos" ou "Checkout". O lint não adivinha.
5. e 6. **Texto literal e referência de outra marca:** ficam fora até existir lista escrita.
7. **Coleção de marca:** parâmetro `brandCollection`. O padrão passa a ser **Tokens APP** quando o documento a tem, senão Tokens WEB.
