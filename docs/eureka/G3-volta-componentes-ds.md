# G3: volta lendo os componentes do DS (Tarefa 7 da FILA)

Branch: `eureka/g3-volta-componentes-ds`, a partir do principal (`claude/eureka-design-lead-epvzr2`).

## O que muda

O spike da Tarefa 6 lia só `div`, `p` e CSS. Uma chamada a componente de código, como `<Button variant="Secundário" />`, se perdia. Agora ela volta como **instância do componente do DS** que tem o link de Code Connect no documento, com variante e propriedades. Nada de camadas desenhadas.

A leitura e o casamento ficam no **motor**, em dois comandos novos (`crates/engine/src/dev/tsx_return.rs`):

- **`codeConnect.readTsx {code | path, components?}`**: só leitura. Para cada chamada a componente de código no TSX, devolve:
  - o componente do documento ligado a ela (pelo nome do componente de código e, se o arquivo importa, pelo pacote do `import`);
  - as propriedades, lidas **invertendo o template do link**: `{Prop}` vira o valor (texto, variante, booleano, instance swap `<Icone />`), e `{Prop ? "a" : "b"}` vira o booleano, como valor de atributo ou como atributo solto;
  - `inside`: o que a instância trocou por dentro. Vem do `components` do resultado do `export.tsx`, se você passar. Senão, vem das notas do cabeçalho do TSX, mas só quando dá para saber de qual chamada é;
  - as notas.
- **`codeConnect.placeCall {call, importPath?, inside?, parent?, x?, y?, name?}`**: põe uma chamada como instância do componente, com a variante e as propriedades. Reaplica as trocas de `inside` (textos, variantes e booleanos de instâncias internas, visibilidade) na camada de mesmo nome. É um passo só de desfazer.

O script (`docs/eureka/spike_tsx_volta.py`) faz só o arranjo:
1. chama o `codeConnect.readTsx` no arquivo;
2. monta a árvore de `div`/`p` como antes;
3. no lugar de cada chamada, põe um `codeConnect.placeCall` com o `inside` lido.

Como uma instância cria camadas, o script passou a usar `{"$step": n}` em vez de prever ids. O documento com os componentes do DS ligados (depois do `codeConnect.import`) entra com `--in`.

### Nomes em código

O nome em código tira o acento e o sufixo de id ("Ícone" vira `icone`, "Texto#7:26" vira `texto`). A volta compara os dois jeitos: o placeholder do template, o atributo da chamada e o nome da camada em `inside` casam pelo nome da propriedade, pelo nome em código ou pela forma sem acento, sem id e sem maiúsculas. O mesmo vale para o valor de variante: `variant="secundario"` vira "Secundário".

Um atributo que o template não passa é lido pelo nome em código da propriedade. Exemplo: `mostrarAcao={false}` vira "Mostrar ação" = false.

### Notas, nunca componente inventado

- **Chamada sem entrada no mapa** (nenhum componente do documento ligado àquele nome): vira nota (`<Chip>: no component of the document is linked to Chip in the code map …`). O `placeCall` não cria nada: `id: null`, nenhuma camada e nenhum passo de desfazer.
- **Prop desconhecida** (o componente não tem essa propriedade): vira nota (`unknown prop tamanho=Grande …`) e fica de fora. As outras propriedades são lidas.
- Também viram nota:
  - um valor que não é variante do componente;
  - um placeholder que mistura texto;
  - um pick que não dá para inverter;
  - um pacote diferente do mapa;
  - mais de um componente ligado ao mesmo nome (vale a leitura com menos notas);
  - uma troca de `inside` sem camada correspondente.

## Como usar

1. `codeConnect.import {path}` no documento do Eureka, com o mapa (Tarefa 3).
2. Volta pelo spike:
   ```
   python -X utf8 docs/eureka/spike_tsx_volta.py --tsx <arquivo.tsx> --snapshot <variables.json> \
     --cli <figcraft-cli> --out <pasta fora do repo> --in <arquivo.figcraft> [--components <resultado do export.tsx>]
   ```
   O relatório ganha `chamadas`, `instancias` e `notas_das_chamadas`.
3. Ou direto pelos comandos: `codeConnect.readTsx {code}`, e depois `codeConnect.placeCall {call, importPath, inside, parent}` para cada chamada.

## Validação

Testes em `crates/engine/src/tsx_return_tests.rs`, com dados sintéticos. Os três falham sem a mudança: os comandos não existem e o teste para em `codeConnect.readTsx: unknown command`.

- `a_screens_design_system_calls_come_back_as_the_same_instances_and_variants`. Uma tela tem um Button do DS (set Hierarquia, na variante Secundário) e um Snackbar do DS. O Snackbar tem texto, instance swap de ícone e booleano, e foi trocado por dentro: a variante do Button de ação e um texto. Os dois estão ligados por `codeConnect.import`. O teste faz `export.tsx` → `readTsx` → `placeCall` num quadro novo. A volta dá:
  - os mesmos componentes, variantes e propriedades;
  - as trocas de dentro reaplicadas;
  - o mesmo texto das chamadas num novo `export.tsx`.

  Sem `components`, o `inside` vem do cabeçalho.
- `a_prop_the_component_does_not_have_is_a_note_and_code_names_are_read`. `tamanho="Grande"` vira nota, `variant="secundario"` vira Secundário e `mostrarAcao={false}` é lido pelo nome em código.
- `a_call_with_no_map_entry_is_a_note_and_nothing_is_made`. `<Chip />` vira nota, sem instância, sem camada e sem passo de desfazer. Um `placeCall` lido é um passo de desfazer. Parâmetros inválidos são recusados.
- Um teste de unidade lê o JSX e o template, e passa lixo nos leitores sem pânico (`dev::tsx_return::tests`).
- Também rodei os filtros `export_tsx`, `code_connect`, `code_map` e `junk`, o `cargo fmt` e o `clippy -D warnings` no engine. O resultado do CI está no commit.
- **Não rodei** o spike ponta a ponta com o arquivo real do DS: o mapa e o snapshot ficam fora do repositório, em `eureka-cache`. Os testes do motor cobrem a leitura e o casamento. O script só arranja a árvore.

## Limites

- **Só React**, a linguagem do `export.tsx`.
- **O template precisa ser um elemento JSX** (`<Nome … />` ou `<Nome …>{prop}</Nome>`). Um template com várias tags ou com código em volta não tem os valores lidos; vira nota.
- **Picks em variante** (`{Tamanho ? "a" : "b"}` sobre uma variante) não dão para inverter: vira nota. Picks em booleano voltam.
- **Slots**: o conteúdo de um slot não volta.
- **`inside`**:
  - o `export.tsx` corta textos em 80 caracteres; um texto desse tamanho volta com nota de que pode estar cortado;
  - a troca vai para a primeira camada com aquele nome que ainda não foi usada, na ordem da árvore;
  - um nome de camada com `": "`, ou um valor com `", "`, pode ser lido errado.
- **Cabeçalho sem `components`**: as notas do cabeçalho dizem a camada, mas não qual chamada. Se só algumas chamadas de um mesmo componente têm trocas, a volta não sabe de qual chamada é cada nota e avisa. Passe o `components` do `export.tsx`.
- **O que o TSX não carrega continua perdido** (ver Tarefa 6): nomes das camadas, os overrides de estilo dentro de uma instância (que o `export.tsx` não escreve), protótipo e metadados do Eureka.
