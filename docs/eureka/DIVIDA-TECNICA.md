# Dívida técnica (Eureka)

Registro de problemas conhecidos, com causa, como confirmar e proposta. Mais novo primeiro.

## DT-1: 3 testes de `board_mindmap_tests` falham sem craft-fonts (Mac), passam no CI

- **Sintoma:** no Mac, 3 testes do módulo `board_mindmap_tests` do crate `figcraft-engine` falham; no CI (Linux) passam.
- **Onde:** `crates/engine/src/board_mindmap_tests.rs`. Os 5 testes de `crates/doc/src/board_mindmap_tests.rs` (geometria pura) passam em todo lugar.
- **Reprodução no Windows** (sem `../craft-fonts`, como o Mac): `cargo test -p figcraft-engine --lib board_mindmap` dá 8 ok e 3 falhas, as mesmas 3:
  - `a_mind_map_is_a_first_node_with_three_children_on_branches`: "child 0: the node is its text, no padding" (largura do nó diferente da do texto).
  - `more_lines_push_the_siblings_apart`: altura medida `(24.0, 24.0)`, esperado `(24.0, 72.0)` (3 linhas não crescem).
  - `two_people_adding_at_once_end_with_one_layout`: nenhuma caixa com altura `3 * h`.
- **Causa provável (com evidência no código):** o tamanho de um nó do mapa é o texto medido (frame "hug", `boards/board/mindmap.rs`, `make_node`; medição em `crates/text/src/layout.rs`). A medição precisa de uma face de fonte: `raw_db()` (`crates/text/src/fontdb.rs`) só carrega as fontes embutidas de `CRAFT_FONTS`, que vêm do build input opcional `CRAFT_FONTS_DIR` (`.cargo/config.toml` aponta para `../craft-fonts`; `crates/text/build.rs`). Sem esse diretório o banco fica vazio até alguém chamar `scan_system_fonts()` (uma vez por processo). Esses testes nunca chamavam o scan: dependiam de outro teste do mesmo processo ter escaneado antes (a ordem muda com o filtro) ou do craft-fonts embutido (o CI tem). O mesmo problema já está documentado em `board_tests.rs` (`a_sticky_keeps_its_text_size_as_text_is_added`, #333), que chama o scan por isso. Sem face, o texto não mede linhas nem largura.
- **Como confirmar no Mac:** `cargo test -p figcraft-engine --lib board_mindmap` deve falhar nos mesmos 3 testes sem a correção e passar com ela. Conferir também se `../craft-fonts` existe ao lado do repositório (se existir e ainda falhar, a causa é outra).
- **Correção (aplicada neste branch):** o módulo de testes define seu próprio `board()` que chama `figcraft_text::scan_system_fonts()` antes de criar a sessão, o mesmo padrão do #333. Só testes mudaram; CI não mudou.
- **Risco restante:** em máquina sem nenhuma fonte de sistema nem craft-fonts os testes ainda falhariam. Uma correção maior seria o motor ter uma fonte embutida mínima para medição em testes (fora do escopo).
