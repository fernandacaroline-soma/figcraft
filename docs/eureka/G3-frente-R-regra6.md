# Frente R: regra 6 do lint, token trocado em instância de componente oficial

Branch `eureka/r-lint-override` (26/11). Faz parte do E17 (G3).

## O problema

A regra 6 da tabela de regras (`G2-F-regras.md`) ainda não entrava no lint. O Fluxo Oficial diz:

> "não trocar nem sobrescrever token de qualquer instância de componente oficial" (Tokens, bloqueio
> crítico)

O Gatekeeper apontou essa falta no parecer da Frente Q. Ela tinha ficado de fora porque o Eureka
não sabia se um componente era oficial. Pela resposta 3 de 24/11, oficial é o que vem de uma
biblioteca do DS, e hoje as importações já marcam isso (`eureka.library`).

## O que muda

O `ds.lint` ganhou a regra **`override-token`** (só aviso; no Notion é bloqueio). Dentro de uma
instância de componente oficial, ela compara cada camada com a camada que ela espelha no componente
principal e aponta:

- **Token trocado:** "`fill` usa X no lugar de Y do componente oficial Button".
- **Token perdido:** "`fill` perdeu o token Y do componente oficial … (valor fixo no lugar)".

**O que não entra:**
- **Componente sem biblioteca registrada:** a regra não adivinha se ele é oficial.
- **Token posto onde o componente tinha valor fixo:** isso não é troca de token.
- **Troca que o próprio componente do DS faz** numa instância dentro dele. O Bottom Sheet e o Action
  Button do DS Apps trocam o fundo das instâncias aninhadas; isso é do DS, não da tela. A comparação
  é feita primeiro com a camada correspondente no componente que contém a instância (`source`). Esse
  caso apareceu na rodada real com 6 falsos positivos, já corrigidos, e está coberto pelo teste.

## Resultado na rodada da NV (`demo-g3r/R-regra6.txt`)

- **As 8 propostas:** lint 0 (nenhuma troca de token).
- **Troca simulada na G:** o fundo do Botão Primário dentro do Action Button foi ligado a
  `Colors/Feedback/Success/Content`. A regra aponta essa troca, com o token novo e o token do
  componente oficial (`Colors/Surface/Brand/Background`). A troca foi feita só no documento local.

## Teste

`the_lint_flags_a_token_swapped_inside_an_official_component_instance` cobre:

- instância intacta;
- componente sem biblioteca;
- token trocado e token perdido;
- token posto onde o componente tinha valor fixo;
- troca feita pelo próprio componente do DS;
- a mesma troca feita na tela.

Sem a correção do `source`, o teste falha, como deveria.

## Rodar

```
python3 docs/eureka/rodada_n.py W eureka-cache/tokens-app-animale-farm-nv-b86217b8e421.json eureka-cache/components-app-frente-p-295d7a197126.json docs/eureka/rodada_nv01_propostas.json docs/eureka/rodada_nv01_disruptivas.json
python3 docs/eureka/regra6_demo.py W <id da tela G>
```
