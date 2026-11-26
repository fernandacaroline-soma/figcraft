# E13: Brand Resolver e perfis de marca

Branch `eureka/e13-brand-resolver` (26/11). Card: E13 no Backlog do Eureka (Notion).

O card pede:
- ler o perfil da marca (modelo `_modelo.md`, com dona e revisão);
- sem perfil, operar só com usabilidade e DS;
- aplicar os tokens e as diferenças estruturais oficiais.

Critérios de aceite: trocar entre duas marcas, avisar das lacunas, manter as versões rastreáveis e não inferir nenhuma regra.

## De onde vêm os dados (só leitura)

A skill gerador-propostas, da Naty, tem:
- o modelo de perfil, `references/marcas/_modelo.md`;
- o perfil da NV, `references/marcas/nv.md`, parcial: a revisão de 12/11 diz "só concorrentes e inspirações";
- o registro de marcas, `references/brand-registry.yaml`, em que todas as outras marcas estão com `perfil: null`.

Copiei o perfil da NV e o registro, só lendo, para `eureka-cache/gerador-propostas/`, fora do repositório. O repositório dela não foi alterado.

Pilotos:
- **NV**, que tem perfil, mas com lacunas;
- **Farm**, que não tem perfil.

## O que entrou

### `brand.profile` (só leitura)

Lê um perfil no formato do `_modelo.md` e devolve:
- **a dona e a data da revisão**, com a observação da revisão como foi escrita;
- **cada seção**, em um destes estados:
  - *preenchida*: o texto vai como está;
  - *a preencher*: a seção só tem marcadores do modelo, como `[a preencher]` ou `- Web: [DS N Marcas / …]`. Nenhum texto é passado adiante;
  - *ponteiro*: "Memória visual" e "Decisões e testes anteriores" apontam para o Notion. Nunca contam como lacuna e nunca são lidas como regra;
- **as lacunas**: perfil sem dona ou sem data, e as seções a preencher;
- **a base de trabalho**:
  - `perfil`, se alguma seção útil estiver preenchida;
  - senão, `usabilidade-e-ds`;
- **uma impressão digital do arquivo**, que mostra qual versão do perfil foi usada.

### `brand.apply` (uma etapa de desfazer)

Para as telas das propostas:
1. **Aplica o modo da marca** na coleção de marca: Tokens APP, ou senão Tokens WEB.
   - O nome pode ser exato ou ter uma observação entre parênteses. "Animale" vira "Animale (CERTO)".
   - Se houver dois modos possíveis, nenhum é escolhido, e a lacuna mostra os dois.
2. **Guarda na tela** com o que ela foi desenhada (`eureka.brand`): o modo, a versão do perfil (dona, revisão, impressão digital), a base de trabalho e as lacunas.
3. **Lista o que o registro de marcas declara oficialmente**: plataformas, arquivo de Team Components, arquivo de Telas e Fluxos e links de benchmark. Um valor `null` no registro vira lacuna.

### No cartão

Uma linha diz para qual marca a proposta foi desenhada, e outra diz quantas lacunas a marca tem:
- com perfil: "Marca: NV · perfil de Nataly, revisão 12/11/2026 (parcial: só concorrentes e inspirações)";
- sem perfil: "Marca: Farm · sem perfil: só usabilidade e DS".

## Teste no app (proposta G, base do template)

Script `docs/eureka/marca_e13.py`, sobre a rodada da NV:

| | NV | Farm |
|---|---|---|
| Modo aplicado | Tokens APP: NV | Tokens APP: Farm |
| Base | perfil (Nataly, 12/11/2026, parcial) | usabilidade e DS (sem perfil) |
| Oficial (registro) | plataformas, Team Components, Telas e Fluxos, concorrentes, referência de marca | plataformas, Team Components, Telas e Fluxos, referência de marca |
| Lacunas | 5 seções a preencher: posicionamento, diferencial, princípios e tom, plataformas e arquitetura de DS, o que não fazer | sem perfil; link de concorrentes pendente no registro |

Prints do cartão: `demo-e13/E13-cartao-NV.png` e `demo-e13/E13-cartao-Farm.png`.

## Validação

- **Motor:**
  - 5 testes: leitura do perfil (dona, revisão, estados das seções, impressão digital diferente por versão);
  - modelo vazio vira só usabilidade e DS;
  - aplicação com registro e desfazer;
  - troca para uma marca sem perfil, modo com parênteses, modo ambíguo e marca sem modo;
  - parâmetros inválidos recusados.
- **Interface:** teste do cartão sem marca, com marca sem perfil e com perfil.
- **Clippy, varredura de parâmetros inválidos e testes de idioma:** passam.

## O que fica para decidir

- **Checkpoint humano obrigatório (card E13):** revisar se a leitura do perfil está certa.
  - "A preencher" vale só para o texto que é marcador do modelo.
  - Ponteiro nunca vira lacuna.
- **Preencher os perfis:** é com a dona de cada marca, na skill da Naty. O Eureka só mostra as lacunas.
- **Ainda não feito:** as diferenças estruturais por marca, como um Team Component que só uma marca tem, não são aplicadas na tela. O registro só diz em qual arquivo elas ficam, e ler esses arquivos é o próximo passo, só com leitura do Figma.
