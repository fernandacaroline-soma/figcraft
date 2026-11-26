# E10: roteiro do teste de usabilidade do loop híbrido

Card E10 no Backlog do Eureka. Entregável: relatório do teste e decisão do Gate 1. **Checkpoint humano obrigatório.**

> Teste feito só por IA **não conta** como validação de usabilidade. A parte A (automática) garante que o produto funciona. Só a parte B (com designers) diz se ele é bom de usar.

## Objetivo

Saber se uma designer consegue trabalhar no Eureka alternando a edição manual com a IA, sem perder controle, componentes e tokens oficiais do Baya DS, e entregar uma tela de qualidade com código reaproveitável.

Os nove pontos a validar:

1. liberdade de edição manual dos componentes;
2. persistência das alterações;
3. facilidade de selecionar e editar elementos;
4. alternância entre edição manual e IA contextual;
5. aplicar, rejeitar e desfazer propostas;
6. preservação dos componentes e tokens oficiais do Baya DS;
7. troca entre marcas;
8. qualidade visual e funcional das entregas;
9. exportação e reaproveitamento do código.

## Parte A: testes automáticos (rodam no CI, sem pessoas)

Eles garantem que cada ponto funciona tecnicamente. **Não medem facilidade nem confiança.**

| Ponto | Já existe (teste automático) | Falta criar |
|---|---|---|
| 1. Edição manual | Comandos de edição com teste no motor (propriedades, auto layout, variantes, instâncias) e a origem por camada: o app sabe o que foi feito pela pessoa e o que veio da IA (G2). | Teste de "editar à mão uma instância do DS sem desanexar": muda texto e variante e confere que continua instância. |
| 2. Persistência | Salvar e reabrir sem perda (`templates_save_and_reopen_losslessly`) e a origem guardada no arquivo. | Salvar, reabrir e conferir a origem e a marca (`eureka.brand`) da proposta. |
| 3. Selecionar e editar | Testes de teclado do app (F6, Tab, camadas) e o cartão alcançável pelo teclado (E14). | Nenhum: facilidade só a designer mede. |
| 4. Manual ↔ IA | Ciclo do G1: a IA vê o que a pessoa mudou (`document.changesSince`), e quando a proposta da IA conflita com uma edição manual, o app pergunta qual fica. | Nenhum novo. |
| 5. Aplicar, rejeitar, desfazer | `proposal.apply`, descartar e desfazer em uma etapa (G1); o agente não aplica no lugar da designer (E15). | Nenhum novo. |
| 6. DS preservado | Lint do DS (regras 1–7), regra 6 (token trocado em instância oficial), regra 7 (componente local) e os snapshots por marca (E14). | Nenhum novo. |
| 7. Troca de marca | `brand.apply` com NV e Farm (E13), snapshots NV × Farm (E14) e contraste nas 9 marcas (V). | Nenhum novo. |
| 8. Qualidade da entrega | Contraste AA, áreas de toque, estados (crítica medida) e o lint do DS. | Nenhum: qualidade percebida só a designer avalia. |
| 9. Código | Export TSX com tokens e componentes do DS (J, Y, Z), mapa de código e volta do código como instâncias (tarefa 7). | Compilar o TSX exportado num projeto React de teste: hoje ninguém confere que ele compila. |

Os três itens "falta criar" (pontos 1, 2 e 9) entram antes da sessão com designers, para não gastar o tempo delas com falha técnica.

## Parte B: sessões com designers (validação de usabilidade)

### Participantes e formato

- **3 designers**, de preferência de marcas diferentes (NV, Farm e uma terceira), que nunca usaram o Eureka.
- **60 a 75 minutos**, moderada, com a designer pensando em voz alta. Gravação de tela só com consentimento.
- **Quem modera:** uma pessoa do time de Design, não a IA. A IA pode preparar o arquivo e anotar, mas não conduz nem avalia.
- **Comparação:** a tarefa 1 também é feita no Figma, com tempo cronometrado, para comparar com e sem Eureka. O card pede tarefas comparáveis.
- **Material:** o app aberto com a rodada da NV (8 propostas sobre o template oficial da PDP), o mapa de código ligado e o perfil das marcas.
- **A IA na sessão:** o Claude conectado ao app (ciclo do G1 e E09). A designer pede em linguagem natural; a prévia e os botões de aplicar ou rejeitar aparecem no app.

### Cenários (tarefas reais de Design)

Cada cenário diz o que pedir à designer, o que conta como sucesso e o que medir.

| # | Tarefa para a designer | Sucesso | Mede os pontos |
|---|---|---|---|
| T1 | "Na PDP da NV, troque o texto do botão principal por 'Comprar agora' e deixe o botão secundário em outra variante. Faça também no Figma." | As duas mudanças feitas, a instância continua oficial (sem desanexar) e nenhum token novo. | 1, 3, 6 |
| T2 | "Salve, feche e abra de novo. Confira se o que você mudou continua lá." | Tudo continua igual depois de reabrir. | 2 |
| T3 | "Peça à IA uma alternativa para a área de compra da proposta que você escolher." Depois: "Aplique uma, rejeite outra e desfaça a que aplicou." | Aplicou, rejeitou e desfez sem ajuda, e sabe dizer o que a IA mudou. | 4, 5 |
| T4 | "Mude à mão algo que a IA acabou de fazer e peça à IA para continuar dali." | A IA respeita a edição manual: pergunta ou mantém o que a designer mudou, sem sobrescrever. | 4 |
| T5 | "No quadro de propostas, escolha a que você levaria adiante, revise a crítica e aprove o layout." | Fez as escolhas pelo cartão e entendeu o parecer do Gatekeeper. | 3, 5 |
| T6 | "Veja essa mesma tela na Farm." | Trocou de marca e notou o aviso de "sem perfil" e as lacunas. | 7 |
| T7 | "Esta tela está pronta para o dev? O que você mudaria?" | A designer avalia a qualidade. Anotar o que ela aponta e comparar com o que a crítica automática apontou. | 8 |
| T8 | "Exporte o código e mostre a um dev (ou à moderadora) o que ele recebe." | Exportou e entendeu as notas do topo do arquivo. Um dev consegue reaproveitar o código sem redesenhar. | 9 |

### Métricas

**Por tarefa:**
- sucesso (completo, parcial ou falha);
- tempo;
- número de erros;
- pedidos de ajuda;
- facilidade de 1 a 7 (SEQ).

**No fim da sessão:**
- **SUS**, a escala de usabilidade de 10 perguntas;
- **confiança de 1 a 5** em três perguntas: "confio que a IA não estraga meu trabalho", "confio que o DS fica preservado" e "levaria esta tela ao handoff";
- três frases curtas: o que mais ajudou, o que mais atrapalhou e o que faltou.

**Comparação:** o tempo da T1 no Eureka contra o tempo no Figma.

### Critérios para o Gate 1 (sugestão, decisão sua)

| Critério | Seguir | Ajustar antes de seguir |
|---|---|---|
| Sucesso nas tarefas | 80% ou mais completas, sem ajuda | Qualquer tarefa com 2 ou mais falhas nas 3 designers |
| SUS | 68 ou mais (média de mercado) | Abaixo de 68 |
| Confiança "a IA não estraga meu trabalho" | média 4 ou mais | Abaixo de 3,5 |
| DS preservado | Nenhuma instância oficial desanexada e nenhum token novo sem querer | Qualquer caso |
| Edição manual × IA | Nenhuma edição manual sobrescrita sem pergunta | Qualquer caso |
| T1 com e sem Eureka | Tempo igual ou menor que no Figma | Mais de 50% mais lento |

### Relatório

O relatório sai depois das 3 sessões, com:
- a tabela de métricas por designer e por tarefa;
- os 5 principais problemas, por gravidade, com trechos da fala das designers;
- recomendações;
- a decisão de seguir ou ajustar, que é sua.

A IA pode consolidar as anotações. Quem avalia a usabilidade são as designers.

## Antes das sessões

1. Criar os três testes automáticos que faltam (pontos 1, 2 e 9).
2. Preparar o arquivo da sessão: a rodada da NV sobre o template, o mapa de código ligado e os perfis de marca.
3. Convidar as 3 designers e marcar a moderadora: decisão sua.
4. Um ensaio rápido com uma pessoa do time, para ajustar o tempo.
