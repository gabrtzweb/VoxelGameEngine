# Voxel Game Engine - Contexto geral do projeto atualmente

Essa é a tentativa de desenvolver uma voxel engine/game experimental, criado do zero por mim usando com Rust e Bevy.

- Bevy 0.19

Escala atual do mundo:

- Bloco lógico: 1x1x1 metro
- Voxels: 0.5x0.5x0.5 metro

A engine concentra-se em um mundo de voxels procedural totalmente editável, com uma estrutura de blocos híbrida, e é inspirada em jogos voxel como Minecraft, mas possui arquitetura e mecânicas próprias.

---

## O que já está funcionando / implementado

O projeto já possui:

- geração procedural de terreno;
- chunks e streaming;
- terreno com variação vertical;
- lagos e corpos de água;
- água atravessável e física básica;
- blocos destrutíveis e colocáveis;
- iluminação dinâmica;
- blocos emissores de luz;
- sombras direcionais;
- fog de distância;
- modos Creative e Spectator;
- movimentação/física básica;
- HUD/debug.

---

## Identidade visual desejada

Não quero visual fotorrealista, mas sim uma estética:

- voxel;
- pixel art;
- estilizada;
- inspirada em jogos voxel.

Não quero:

- circulos perfeitos.
- modelos não cuboides.
- fotorrealismo.

Quero iluminação e atmosfera inspiradas em shaders/Vibrant Visuals, mas mantendo estética voxel/pixel art.

---

## O que quero implementar agora

- textura individual por voxel
- sol pixelado/estilizado.
- lua pixelada/estilizada.
- ciclo completo de dia e noite, dividido em 4 fases:
 manhã.
 meio-dia.
 entardecer.
 noite.
- implementar as 4 fases da Lua (pode ser alterado uma fase por dia).
- sistema de nuvens

## Regras importantes ao ajudar

- Inspecionar primeiro os arquivos atuais antes de substituir sistemas.
- Manter a engine funcional durante a migração.
- Validar alterações com `cargo fmt`, `cargo check` e `cargo clippy`.
- Não adicionar comentários desnecessários no código.
- Manter o código 100% em inglês, sem nenhum trecho ou comentários em português.
