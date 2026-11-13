---
name: eureka-ux-ai
description: UX e interação com IA do Eureka Design. Use sempre que for projetar, revisar ou alterar qualquer experiência visual do Eureka (painéis, canvas, fluxos com Claude, conectores de agentes, aprovações), ou quando alguém pedir "proposta de interação", "protótipo de fluxo com IA", "como o designer chama o Claude", "revisão de UX". Garante edição visual como centro, IA com escopo/preview/diff/undo, governança do Design System e decisão humana nos gates.
---

# Eureka UX & AI Interaction

Rascunho v0 (Sprint 0). Dona das decisões de produto e UX: a liderança de Design do Eureka. Esta skill
orienta propostas; ela não aprova nenhuma direção de UX sozinha.

## Princípios (verifique cada proposta contra todos)

1. **O canvas é o centro.** A edição visual direta é o caminho principal. O chat é um meio, não a
   interface. Nenhum ajuste simples (cor, espaço, texto, ordem) deve exigir um prompt.
2. **A IA vê o que o designer fez.** Antes de cada pedido, o estado atual do documento (incluindo
   edições manuais) é a fonte. A IA nunca regenera do zero por cima de trabalho manual; ela propõe
   comandos sobre o documento existente.
3. **Toda ação da IA tem escopo explícito.** Mostre sobre o que ela vai agir (seleção, frame,
   página) antes de rodar. Fora do escopo, ela pede permissão.
4. **Proposta antes de aplicar.** Mudanças da IA chegam como proposta: preview no canvas (fantasma
   ou lado a lado), lista do que muda (diff legível por designer, não JSON), Aceitar / Rejeitar /
   Ajustar. Aceitar vira um único passo de desfazer.
5. **Execução visível e interrompível.** Mostre progresso, onde a IA está agindo, e um Cancelar
   que funciona. Erros dizem o que falhou e o que fazer, sem desculpas vagas.
6. **Design System primeiro.** Gere e edite com tokens, componentes e variantes oficiais. Valor
   solto (hex, px sem token) é sinalizado, nunca escondido. A IA não cria nem altera tokens,
   componentes ou regras globais: ela aponta a lacuna e encaminha para governança humana.
7. **Marca com fonte.** Use apenas perfis de marca preenchidos (responsável + data de revisão).
   Sem perfil, diga a limitação e trabalhe só com usabilidade e DS.
8. **Agentes independentes, Eureka como cockpit.** Gatekeeper, Builder, Docs, Audit e agentes de
   exploração rodam fora; o Eureka chama, mostra estado, resultado e pede decisão.
9. **Humano decide nos gates.** Escolha entre propostas, registro no Notion, exclusões no Figma,
   merge e mudança de DS sempre pedem confirmação explícita.
10. **Não interrompa.** Sugestões não abrem modais nem roubam foco; ficam disponíveis até o designer
    olhar.

## Checklist de uma proposta de UX

- [ ] Fluxo descrito do ponto de vista do designer (o que ele vê e faz, passo a passo).
- [ ] Como a IA é chamada (atalho, seleção, painel) e de onde o contexto vem.
- [ ] Estados: vazio, executando, proposta pronta, erro, cancelado, aplicado, desfeito.
- [ ] Como o diff aparece e como rejeitar parte dele.
- [ ] O que acontece com edições manuais feitas durante a execução.
- [ ] Onde o Gatekeeper de DS entra e o que bloqueia.
- [ ] Acessibilidade: teclado completo, foco visível, contraste, leitor de tela para propostas.
- [ ] Protótipo visual (screenshot do app ou mock) anexado.
- [ ] Riscos e o que precisa de decisão humana.

## Fluxo de exploração (preservar)

Problema/hipótese → 3 a 5 propostas com evidências (benchmarking, usabilidade, inovação) →
escolha humana → crítica de Design → DS Gatekeeper → refinamento. O Notion guarda o estudo e todas
as alternativas; o Figma recebe só a escolhida.

## Anti-padrões

- Interface que só funciona por chat.
- Regenerar a tela inteira para mudar um detalhe.
- Aplicar mudanças da IA sem preview, ou com um desfazer por passo interno.
- Esconder valores fora do DS ou inventar tokens.
- Inventar características de marca.
- Diálogos modais para sugestões.
