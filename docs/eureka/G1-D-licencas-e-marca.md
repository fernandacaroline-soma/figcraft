# G1-D — Licenças e risco de marca

Auditoria somente leitura. Branch `claude/eureka-design-lead-epvzr2`. Nada foi compilado; só `cargo metadata --format-version 1 --locked` (passou, 670 pacotes incluindo os do workspace).

## Resumo

- Os 22 crates do workspace são `MIT OR Apache-2.0`. Nenhum dependente de terceiros está sem campo `license`.
- 41 pacotes de terceiros (35 crates distintos) têm expressão que NÃO fecha só com a lista do AGENTS.md. Os identificadores fora da lista são `Unicode-3.0`, `Zlib`, `MPL-2.0` / `MPL-2.0+`, `BSL-1.0` e `CDLA-Permissive-2.0`. O repo não tem `deny.toml` nem outra configuração de cargo-deny, então nada declara exceção. A lista do AGENTS.md é a única regra.
- O Eureka não adicionou dependência: `git diff origin/main...HEAD` (63 arquivos, 21 commits) não toca em nenhum `Cargo.toml` nem no `Cargo.lock`.
- Marca: os logos ArtCraft de `docs/brand/` NÃO são carregados pelo binário. O que aparece ao usuário é "FigCraft" (janela, ícone do app, Sobre) e texto/links "ArtCraft" no Sobre, no instalador e nas traduções. A licença da marca manda remover ou substituir as marcas numa versão modificada distribuída.
- Risco operacional extra: o atualizador embutido aponta para `storytold/figcraft` e pode oferecer ao usuário interno a versão upstream, que substituiria a build Eureka.

## Licenças

Critério: expressão SPDX satisfeita apenas com identificadores da lista. `A OR B` passa se um lado é totalmente permitido; `AND` exige os dois. A análise cobre todos os alvos (inclui crates só de Windows, Linux e WASM), então é um teto: nem todos entram numa build macOS. Quais entram de fato não foi verificado (exigiria `cargo tree`, que não rodei).

| Identificador fora da lista | Crates (versão) | Observação |
|---|---|---|
| `Unicode-3.0` (puro) | fixed_decimal 0.7.3, icu_collections, icu_decimal, icu_locale_core, icu_locale_fallback, icu_locale_fallback_data, icu_normalizer, icu_normalizer_data, icu_pattern 0.5.1, icu_plurals, icu_properties, icu_properties_data, icu_provider 2.3.1, icu_provider_adapters, icu_provider_blob, icu_segmenter, icu_segmenter_data (todos 2.3.x salvo indicado), litemap 0.8.3, potential_utf 0.1.6, tinystr 0.8.5, writeable 0.6.4, yoke 0.8.4, yoke-derive 0.8.4, zerofrom 0.1.8, zerofrom-derive 0.1.8, zerotrie 0.2.5, zerovec 0.11.9, zerovec-derive 0.11.7 | Família ICU4X/unicode-org. Licença permissiva, mas não listada. |
| `(MIT OR Apache-2.0) AND Unicode-3.0` | unicode-ident 1.0.27 | O `AND` obriga a cumprir também Unicode-3.0. |
| `Zlib` | foldhash 0.1.5 e 0.2.0, slotmap 1.1.1, zlib-rs 0.6.8 | Permissiva. |
| `MPL-2.0` / `MPL-2.0+` | option-ext 0.2.0, imbl 7.0.2, imbl-sized-chunks 0.2.0 | Copyleft fraco por arquivo. É o único grupo que exige análise real. |
| `BSL-1.0` | clipboard-win 5.4.3, error-code 3.4.0 | Permissiva (provavelmente só Windows). |
| `CDLA-Permissive-2.0` | webpki-roots 1.0.11, webpki-root-certs 1.0.11 | Licença de dados (certificados raiz Mozilla). |

Com exceção (reportado à parte): `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT` em linux-raw-sys 0.4.15 e 0.12.1, rustix 0.38.44 e 1.1.5, wasi 0.11.1, wasip2 1.0.4, wasm-encoder 0.262.0, wasmparser 0.228.0 e 0.262.0, wast 262.0.0, wat 1.262.0, wit-bindgen 0.57.1. Fecham pelo lado `Apache-2.0 OR MIT`, então não violam a lista; a exceção LLVM nunca precisa ser invocada.

Sem campo `license`: nenhum (nenhum caso com `license_file`).

Fora do grafo de crates, não verifiquei fontes e assets (fontes ficam em `storytold/craft-fonts`, fora do repo). Vale notar que `epaint_default_fonts` é `(MIT OR Apache-2.0) AND OFL-1.1 AND Ubuntu-font-1.0`, tudo na lista.

Configuração do repo: não existe `deny.toml`, `about.toml` nem job de cargo-deny (`git ls-files` e `find`). Logo "o que o repo declara" = só o AGENTS.md, e o item 9 das house rules diz para parar e perguntar sobre licença fora da lista. Se existir exceção para `Unicode-3.0`/`Zlib` etc., ela estaria no `craft-contrib` ou no `craft-common-actions` (CI compartilhado), que não consultei. Pelo AGENTS.md, um repo "não pode adicionar identificador editando a própria cópia".

## Dependências novas do Eureka

Nenhuma. Merge-base com `origin/main` = `011f08e`. O diff `origin/main...HEAD` tem 5198 inserções em 63 arquivos, todos em `docs/`, `.claude/skills`, `.github/workflows/eureka.yml`, `crates/engine`, `crates/mcp`, `crates/ui-egui` (src e locales); nenhum `Cargo.toml` ou `Cargo.lock`. Atenção: a branch local `main` aponta para outro commit (`b5b4fb8`, igual a `fork/main`) e NÃO é o upstream; um diff contra ela mostra 1042 arquivos e não serve para esta pergunta. `.github/workflows/eureka.yml` é um workflow novo (o AGENTS.md pede não mexer em CI em PR de feature; relevante só se algum dia houver PR ao upstream).

## Marca

O que o usuário final de uma build interna veria. Os 8 arquivos de `docs/brand/` só são referenciados por README.md (URLs remotas, linhas 4-5 e 377) e ATTRIBUTION.md:15. Nenhum `include_bytes!`/asset do app aponta para eles.

| Onde aparece | Arquivo:linha | O que trocar |
|---|---|---|
| Título da janela principal | `apps/figcraft/src/main.rs:263` ("FigCraft") | string |
| Título dos diálogos de erro | `apps/figcraft/src/startup.rs:24`; `crates/ui-egui/src/windows.rs:195` | string |
| app_id Wayland/X11 | `apps/figcraft/src/main.rs:264` (`ai.storyteller.figcraft`) | string (e os arquivos do Linux abaixo, que precisam bater) |
| Wordmark "Fig"+"Craft" no Sobre (desenhado em código, não é o logo ArtCraft) | `crates/ui-egui/src/theme.rs:423-429`, usado em `crates/ui-egui/src/dialogs.rs:147` | strings "Fig" / "Craft" |
| Texto do Sobre: "part of the ArtCraft Crafting Apps", "Made by the ArtCraft team and community" | `crates/ui-egui/src/dialogs.rs:155,162` e a chave idêntica em cada `crates/ui-egui/locales/*.toml` (en.toml:524,780; pt-BR.toml e as demais) | texto-fonte + chaves nas 26 locales (a chave é o texto em inglês) |
| Links do Sobre: getartcraft.com, Discord artcraft, issues storytold | `dialogs.rs:44,158,159`; `app/commands.rs:231` (Ajuda) | URLs |
| Strings "FigCraft" em locales (ex.: "Sobre o FigCraft", pt-BR.toml:25,107,499...) | `crates/ui-egui/locales/*.toml` (26 arquivos), `crates/ui-egui/src/dialogs.rs:53,77`, `services.rs:149`, `dialogs/import_report.rs:161` | texto-fonte + todas as locales; `pt-BR.toml` é a que a equipe usa |
| Ícone do app (camaleão, original do dono, MIT/Apache; NÃO é marca ArtCraft) | `assets/app-icon/` (figcraft.svg, *.png, .ico, hicolor/), embutido em `apps/figcraft/src/services.rs:576-578`, `crates/ui-egui/src/lib.rs:12`, `theme.rs:435`, `apps/figcraft/build.rs:5-9` | Opcional para a licença. Recomendável trocar para não parecer o FigCraft oficial. Regerar com `cargo xtask icon`; nomes de arquivo `ai.storyteller.figcraft.png` e `figcraft-*.png` são referenciados em vários pontos |
| Pasta de dados/config do usuário | `apps/figcraft/src/services.rs:32` (`APP_ID = "FigCraft"`) | Trocar separa configurações da instalação do FigCraft oficial; invalida configs existentes |
| Atualizador automático | `crates/ui-egui/src/lib.rs:11-14` (`..craft_update::suite::FIGCRAFT`, repo `storytold/figcraft`, definido no crate git `craft-update`) | Decisão necessária: desativar a checagem ou apontar para outro repo. Não é só texto. Ver Riscos |
| macOS: nome, executável, bundle id, esquema de URL | `packaging/macos/Info.plist.in:9-16,100,103` (`ai.storyteller.figcraft`, esquema `figcraft://`), `packaging/macos/package.sh:4-5,42-43,104-117,153,174`, UTI `...figcraft.document` (Info.plist.in:53,120) | strings + nome do DMG; o esquema `figcraft://` e a extensão `.figcraft` são o formato de arquivo, tratar à parte |
| Windows MSI | `packaging/windows/figcraft.wxs:6-7,11,18-19,23-24,31,55-56` (Name, Manufacturer "Learning Machines LLC", pasta `ArtCraft\FigCraft`, URLs), `packaging/windows/check-install-policy.ps1:58-91,164-174`; trocar também o `UpgradeCode` (wxs:6) para não substituir o instalador oficial | strings + GUID |
| Linux (.deb/.rpm/Flatpak/AppImage) | `packaging/linux/nfpm.yaml:7,17-18`, `ai.storyteller.figcraft.desktop`, `.metainfo.xml.in:5,10`, `.mime.xml`, `flatpak/*.yml` (id e `xdg-data/ArtCraft/versions`), `package.sh:26,52` (mantenedor `figcraft@storyteller.ai`) | renomear arquivos e ids |
| FreeBSD / web / servidor | `packaging/freebsd/package.sh:20`, `packaging/web/`, `packaging/server/` | só se forem distribuídos |
| Crate/binário | `apps/figcraft/Cargo.toml:2,7` (`name = "figcraft"`) e o nome dos 22 crates `figcraft-*` | NÃO trocar: não aparece ao usuário e gera churn enorme |
| Documentação e licença | `README.md:4-5,371-377`, `NOTICE:6-8`, `ATTRIBUTION.md:15`, `docs/brand/*` | Para distribuição interna sem logos: remover `docs/brand/` ou não empacotá-los; manter o texto em simples dizendo "baseado em FigCraft da equipe ArtCraft", que a licença permite |

Ordem do menor para o maior conjunto, se a meta é só estar dentro da licença de marca:

1. Não distribuir `docs/brand/` nem o README com os logos (nenhuma mudança de código; os logos não estão no binário).
2. Texto/links "ArtCraft" do Sobre (`dialogs.rs:155-162`) mais a chave nas locales, e o nome "FigCraft" visível: `main.rs:263`, `startup.rs:24`, `windows.rs:195`, wordmark `theme.rs:423-429`.
3. Atualizador (`lib.rs:11-14`): desativar ou redirecionar.
4. Empacotamento do macOS (`Info.plist.in`, `package.sh`), que é o alvo provável da equipe.
5. Ícone, `APP_ID` e demais plataformas.

## Riscos e recomendação

- Marca: a licença (`docs/brand/LICENSE-brand.txt`, seção "Forks and modified versions") obriga, ao distribuir versão modificada, remover ou substituir as marcas e não chamar de ArtCraft. "FigCraft" em si não está coberto por essa licença de marca (ela fala em ArtCraft), mas o Sobre diz "made by the ArtCraft team", o que pode sugerir que a ArtCraft mantém a build Eureka. Se a distribuição interna é considerada distribuição, trate como risco. Não fui eu quem deve decidir isso: confirmar com jurídico/liderança.
- O ícone do camaleão é MIT/Apache (`assets/app-icon/LICENSE.txt`), então pode ser mantido legalmente, mas faz a build parecer o FigCraft oficial.
- Atualizador: um usuário interno clicando em "Check for updates" (ou com a checagem automática) pode ser levado a instalar o FigCraft upstream por cima. Se a checagem de inicialização vem ligada por padrão, não verifiquei (`updates_ui.rs:37` inicializa `startup_check: false` no runtime, mas o valor vem das configurações).
- Licenças: as não listadas são permissivas, exceto MPL-2.0 (option-ext, imbl, imbl-sized-chunks), que tem obrigação por arquivo modificado (basta não modificar esses crates). Recomendação: perguntar ao responsável se há exceção oficial para `Unicode-3.0`, `Zlib`, `BSL-1.0`, `CDLA-Permissive-2.0` e `MPL-2.0` (pelo item 9, não adicionar por conta própria) e rodar `cargo tree -e normal --target aarch64-apple-darwin` fora desta sessão para saber quais entram na build de macOS. Um `deny.toml` com essas exceções registradas tornaria a decisão auditável.
- Não verificado: se existe exceção no `craft-contrib`; o grafo real por alvo; o conteúdo dos textos de licença dos crates; o que o crate `craft-update` faz exatamente além de apontar para `storytold/<app>/releases` (li só `suite.rs` e `app.rs`); o comportamento das builds web e servidor; strings de marca dentro de `docs/` e `plugins/` (excluí essas pastas da busca por código).
