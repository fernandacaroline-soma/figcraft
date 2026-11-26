# G3: TSX com os componentes do DS (Tarefa 2 da FILA)

Branch: `eureka/g3-tsx-ds-components`. Ele parte de `eureka/g3-tokens-web` (Tarefas 1 e 4) e recebe o merge de `eureka/g3-code-map` (Tarefa 3). Para juntar no principal, basta juntar este branch: ele leva as quatro tarefas.

## O que muda

Antes, o `export.tsx` redesenhava toda instância com `div` e CSS, inclusive os componentes do DS. Agora, numa instância cujo componente tem link de Code Connect em React:

```tsx
import { Button } from "@pacote/ds-apps";
/* cabeçalho da tela */
import "./pdp-mobile.css";
…
        <Button variant="Secundário" />
```

- **A chamada usa o template do mapa**, com os valores da própria instância. É o mesmo template que o `inspect.code` e o Dev Mode já usam.
- **O `import` vem do `importPath` do link.** Esse é o "pacote configurável": muda no mapa, não no código.
- **O que está dentro do componente não entra no export:** nem CSS nem avisos de valor solto. Essa parte é do componente de código e do time de DS.
- **A saída ganha `components`:** `[{id, layer, component, name, importPath}]`, as chamadas geradas.

Uma instância de biblioteca do DS (`eureka.library`) **sem** link continua desenhada, como antes, e vira nota no resultado e no cabeçalho. Exemplo: `Chip: Chip is a design-system component with no code component in the map (drawn as layers)`.

Uma instância de componente local, que não é do DS, continua desenhada, sem nota.

## Como usar

1. Prepare o mapa em `eureka-cache/`. O formato está na Tarefa 3, em `docs/eureka/G3-code-map.md`; há um modelo em `~/eureka-cache/ds-apps.connect.json`.
2. `export.codeMap {path}`: confere o mapa.
3. `codeConnect.import {path}`: liga os componentes do documento.
4. `export.tsx` ou o botão **Exportar código**: o TSX sai com as chamadas e os `import`.

## Validação

- `a_design_system_component_in_the_map_is_a_call_to_its_code_component`. Uma tela com um Button do DS ligado pelo mapa, na variante Secundário com preenchimento solto dentro do componente, e um Chip do DS sem link. Resultado:
  - o `import` abre o arquivo e a chamada traz o valor da instância;
  - nenhum valor de dentro do componente entra, nem como aviso;
  - `components` traz a chamada;
  - a nota do Chip aparece no resultado e no cabeçalho.
- `without_a_map_the_design_systems_instances_are_drawn_and_noted`. Sem o link, o Button é desenhado e anotado, e `components` fica vazio.
- `two_screens_with_one_code_component_import_it_once_at_the_top`. Duas telas com Button no mesmo `.tsx`: um único `import { Button }`, no topo, e duas chamadas.
- `a_layer_named_as_an_imported_component_gets_a_number`. Uma camada raiz chamada "Button" junto de uma tela que importa o Button vira `function Button2`, com nota.
- Também rodei os filtros `code_connect`, `code_map`, `codegen`, `inspect`, `eureka`, o junk sweep e o `clippy -D warnings` no engine. O resultado do CI está no commit.

## Limites

- **Só React.** É a linguagem do `export.tsx`. Os links de outras linguagens ficam para outros exports.
- **Template e chamada.** Uma propriedade que o template do mapa não lê não chega ao código; o `export.codeMap` lista essas propriedades em `unmapped`. Um slot preenchido também não aparece na chamada, a menos que o template o leia.
- **Componentes aninhados.** Uma instância ligada dentro de outra instância não ligada aparece como chamada no lugar dela, e o `import` sobe para o topo. Uma instância dentro de uma instância ligada não é vista, porque o componente de código cuida dela.
- **Sem conferência no código.** Não há verificação de que o pacote ou a prop existem. Ver os limites da Tarefa 3.

## Ajustes da revisão (PC1, 26/11)

- **Imports no topo, sem repetição.** Antes, cada tela trazia os próprios `import`, e duas telas com Button davam dois `import { Button }` no mesmo arquivo (identificador duplicado). Agora os `import` de todas as telas são juntados, sem repetição, e abrem o arquivo.
- **Nome de camada igual ao de um componente importado.** Uma camada raiz com o nome de um componente importado (por exemplo, "Button") ganha número (`Button2`), como duas camadas de mesmo nome. A nota diz `Button is also an imported code component: this layer (…) is Button2`.
- O merge com `g3-tokens-web` tinha perdido um `}` em `export_tsx_tests.rs`; está corrigido.
