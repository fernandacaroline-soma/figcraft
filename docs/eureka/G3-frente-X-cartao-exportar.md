# Frente X: "Exportar a proposta como código" no cartão

Branch `eureka/x-cartao-exportar` (26/11). Liga ao cartão do quadro o comando `explore.exportCode`
que o outro computador fez (tarefa 5 da fila).

## O que muda

O botão **"Exportar a proposta como código"** aparece no cartão depois do parecer do Gatekeeper,
junto de "Aprovar o layout".
- **No desktop:** pede a pasta e grava o `.tsx` da tela e o `tokens.css`.
- **No navegador:** baixa os dois arquivos.
- **Ao terminar:** um aviso diz quantos valores ficaram sem token.

O TSX traz no topo um resumo para os devs:
- a proposta;
- o parecer, com cada item;
- os achados da crítica que continuam abertos, por severidade.

## Teste no app (G, base do template)

`explore.exportCode` gerou `GDisruptivaComprarAgoraNoTamanhoDeSempre.tsx` e `tokens.css`: 51 tokens e
56 valores sem token. Os valores sem token vêm das peças do template que o mapa de código ainda não
cobre e que saem desenhadas como camadas.

- **`demo-g3x/X0-cabecalho-tsx-G.txt`:** o cabeçalho do TSX, só com nomes.
- **`X1`:** o cartão com o botão.

O `tokens.css` tem os valores do DS e **não entra no repositório**.
