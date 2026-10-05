# Catálogo e Estrutura de Worldbuilding: Solos e Terrenos

Todas as texturas ativas foram organizadas na pasta do projeto:
📂 **`C:\docs\worldbuilding\soils`**

Overlays compartilhados de superfície estão na pasta:
📂 **`C:\docs\worldbuilding\overlays`**

---

## 1. Catálogo Completo de Texturas em worldbuilding/soils/ (66 Texturas)

### A. Solos Biológicos Principais (10 variantes cada = 30 texturas)
Para cada tipo de solo (**Loam**, **Silt**, **Peat**):
- **`dirt`**: Bloco base de solo puro.
- **`coarse`**: Solo áspero com seixos embutidos.
- **`grass_side`**: Lateral com cobertura de grama.
- **`mud`**: Solo encharcado e lamoso.
- **`mulch`**: Camada superior com serapilheira e folhas caídas.
- **`mulch_side`**: Lateral com caimento de folhas.
- **`rooted`**: Solo entrelaçado com raízes expostas (máscara preservada).
- **`snow_side`**: Lateral com camada de neve.
- **`tilled`**: Solo preparado e arado para cultivo agrícola.
- **`tilled_side`**: Lateral do solo arado com ranhura de cultivo.

### B. Solos Climáticos (4 variantes cada = 12 texturas)
- **Permafrost (Frio/Polar)**:
  - [``soil_permafrost_dirt.png``] (Solo base congelado com micro-cristais de gelo)
  - [``soil_permafrost_grass_side.png``] (Lateral com cobertura de grama)
  - [``soil_permafrost_coarse.png``] (Solo áspero com cascalho/gelo embutido)
  - [``soil_permafrost_icy.png``] (Solo maciço com veios de gelo glacial azul — versão _alt disponível)
- **Caliche (Quente/Árido)**:
  - [``soil_caliche_dirt.png``] (Solo base alcalino de deserto)
  - [``soil_caliche_grass_side.png``] (Lateral com cobertura de grama)
  - [``soil_caliche_coarse.png``] (Solo áspero com seixos embutidos)
  - [``soil_caliche_cracked.png``] (Solo rachado pela seca extrema)
- **Laterite (Quente/Úmido Tropical)**:
  - [``soil_laterite_dirt.png``] (Solo base de terra roxa tropical rica em óxidos de ferro)
  - [``soil_laterite_grass_side.png``] (Lateral com cobertura de grama)
  - [``soil_laterite_coarse.png``] (Solo com pisólitos de ferro/concreções)
  - [``soil_laterite_hardpan.png``] (Couraça de ferro laterítica endurecida / ferricrete)
### C. Clays / Argilas (4 texturas)
- [``soil_clay_gray.png``] (Aluvial comum)
- [``soil_clay_red.png``] (Terracota rica em ferro)
- [``soil_clay_white.png``] (Caulim puro para cerâmica fina)
- [``soil_clay_yellow.png``] (Ocre com limonita)

### D. Sands / Areias (6 texturas)
- [``soil_sand.png``] (Comum quartzosa)
- [``soil_sand_red.png``] (Desértica avermelhada)
- [``soil_sand_dune.png``] (Dunas douradas)
- [``soil_sand_white.png``] (Praia de gipsita/coral)
- [``soil_sand_pink.png``] (Areia rosa de conchas/foraminíferos)
- [``soil_sand_black.png``] (Areia basáltica vulcânica)

### E. Gravels / Cascalhos (Sistema Matriz - 4 texturas)
- [``soil_gravel_dirty.png``] (Matriz terrosa de loam)
- [``soil_gravel_sandy.png``] (Matriz arenosa fluvial)
- [``soil_gravel_ashy.png``] (Matriz de cinza vulcânica escura)
- [``soil_gravel_snowy.png``] (Matriz congelada com gelo e neve)

### F. Cinzas Vulcânicas (2 texturas)
- [``soil_ash_vulcanic.png``] (Cinza escura basáltica)
- [``soil_ash_pumice.png``] (Pó de pedra-pomes clara e porosa)

### G. Musgos de Solo (4 texturas)
- [``soil_moss_forest.png``] (Musgo aveludado de florestas temperadas, riachos e bosques.)
- [``soil_moss_crimson.png``] (*Sphagnum rubellum/magellanicum* das turfeiras ácidas e tundras boreais.)
- [``soil_moss_amber.png``] (*Tomenthypnum nitens* (*Golden Feather*) de clareiras ensolaradas e pântanos.)
- [``soil_moss_cave.png``] (*Schistostega pennata* (*Goblin Gold*) que reflete luz em cavernas e grutas.)

### H. Neves & Grama (4 texturas)
- [``soil_snow.png``] (Topo e bloco sólido)
- [``soil_snow_powder.png``] (Neve fofa aerada)
- [``soil_grass.png``] (Topo em escala de cinza para biome tint)
- [``soil_grass_colored.png``] (Topo pré-colorido)

---

## 2. Catálogo Completo de Blocos (106 Blocos)

Este catálogo define a composição de faces (topo, fundo e laterais) e o uso de overlays para cada bloco do jogo.

---

### A. Solos Biológicos (Loam, Silt, Peat)

Cada um dos três solos biológicos possui 8 blocos construtíveis:

#### 1. Loam (Solo Temperado Balanceado)
- **Loam Dirt**: `soil_loam_dirt.png` em todas as 6 faces.
- **Loam Coarse**: `soil_loam_coarse.png` em todas as 6 faces.
- **Loam Mud**: `soil_loam_mud.png` em todas as 6 faces.
- **Loam Rooted**: `soil_loam_rooted.png` em todas as 6 faces.
- **Loam Grass**: `soil_grass.png` (em escala de cinza) no topo, `soil_loam_dirt.png` no fundo, e na lateral `soil_loam_grass_side.png` + `soil_grass_side_overlay.png` por cima (em escala de cinza, somente na parte da grama).
- **Loam Snow**: `soil_snow.png` no topo, `soil_loam_dirt.png` no fundo, e na lateral `soil_loam_snow_side.png`.
- **Loam Mulch**: `soil_loam_mulch.png` no topo, `soil_loam_dirt.png` no fundo, e na lateral `soil_loam_mulch_side.png`.
- **Loam Tilled**: `soil_loam_tilled.png` no topo, `soil_loam_dirt.png` no fundo, e na lateral `soil_loam_tilled_side.png`.

#### 2. Silt (Solo Fino Aluvial / Fluvial)
- **Silt Dirt**: `soil_silt_dirt.png` em todas as 6 faces.
- **Silt Coarse**: `soil_silt_coarse.png` em todas as 6 faces.
- **Silt Mud**: `soil_silt_mud.png` em todas as 6 faces.
- **Silt Rooted**: `soil_silt_rooted.png` em todas as 6 faces.
- **Silt Grass**: `soil_grass.png` (em escala de cinza) no topo, `soil_silt_dirt.png` no fundo, e na lateral `soil_silt_grass_side.png` + `soil_grass_side_overlay.png` por cima (em escala de cinza, somente na parte da grama).
- **Silt Snow**: `soil_snow.png` no topo, `soil_silt_dirt.png` no fundo, e na lateral `soil_silt_snow_side.png`.
- **Silt Mulch**: `soil_silt_mulch.png` no topo, `soil_silt_dirt.png` no fundo, e na lateral `soil_silt_mulch_side.png`.
- **Silt Tilled**: `soil_silt_tilled.png` no topo, `soil_silt_dirt.png` no fundo, e na lateral `soil_silt_tilled_side.png`.

#### 3. Peat (Solo Orgânico Escuro de Pântano / Turfeira)
- **Peat Dirt**: `soil_peat_dirt.png` em todas as 6 faces.
- **Peat Coarse**: `soil_peat_coarse.png` em todas as 6 faces.
- **Peat Mud**: `soil_peat_mud.png` em todas as 6 faces.
- **Peat Rooted**: `soil_peat_rooted.png` em todas as 6 faces.
- **Peat Grass**: `soil_grass.png` (em escala de cinza) no topo, `soil_peat_dirt.png` no fundo, e na lateral `soil_peat_grass_side.png` + `soil_grass_side_overlay.png` por cima (em escala de cinza, somente na parte da grama).
- **Peat Snow**: `soil_snow.png` no topo, `soil_peat_dirt.png` no fundo, e na lateral `soil_peat_snow_side.png`.
- **Peat Mulch**: `soil_peat_mulch.png` no topo, `soil_peat_dirt.png` no fundo, e na lateral `soil_peat_mulch_side.png`.
- **Peat Tilled**: `soil_peat_tilled.png` no topo, `soil_peat_dirt.png` no fundo, e na lateral `soil_peat_tilled_side.png`.

---

### B. Solos Climáticos

#### 1. Permafrost (Frio / Polar / Tundra)
- **Permafrost Dirt**: `soil_permafrost_dirt.png` em todas as 6 faces.
- **Permafrost Coarse**: `soil_permafrost_coarse.png` em todas as 6 faces.
- **Permafrost Grass**: `soil_grass.png` (em escala de cinza) no topo, `soil_permafrost_dirt.png` no fundo, e na lateral `soil_permafrost_grass_side.png` + `soil_grass_side_overlay.png` por cima (em escala de cinza, somente na parte da grama).
- **Permafrost Icy**: `soil_permafrost_icy.png` em todas as 6 faces (solo maciço cortado por veios de gelo glacial puro).

#### 2. Caliche (Árido / Desértico / Chaparral)
- **Caliche Dirt**: `soil_caliche_dirt.png` em todas as 6 faces.
- **Caliche Coarse**: `soil_caliche_coarse.png` em todas as 6 faces.
- **Caliche Cracked**: `soil_caliche_cracked.png` em todas as 6 faces.
- **Caliche Grass**: `soil_grass.png` (em escala de cinza) no topo, `soil_caliche_dirt.png` no fundo, e na lateral `soil_caliche_grass_side.png` + `soil_grass_side_overlay.png` por cima (em escala de cinza, somente na parte da grama).

#### 3. Laterite (Quente / Úmido / Florestas Tropicais)
- **Laterite Dirt**: `soil_laterite_dirt.png` em todas as 6 faces.
- **Laterite Coarse**: `soil_laterite_coarse.png` em todas as 6 faces.
- **Laterite Hardpan**: `soil_laterite_hardpan.png` em todas as 6 faces (couraça endurecida de pedra de ferro).
- **Laterite Grass**: `soil_grass.png` (em escala de cinza) no topo, `soil_laterite_dirt.png` no fundo, e na lateral `soil_laterite_grass_side.png` + `soil_grass_side_overlay.png` por cima (em escala de cinza, somente na parte da grama).

---

### C. Musgos de Solo (Moss Blocks)

Os blocos de musgo atuam tanto como blocos sólidos de terreno biológico quanto fornecem overlays para cobrir solos e rochas:

- **Forest Moss**: `soil_moss_forest.png` em todas as 6 faces. *(Overlay lateral para solo/rocha: `moss_forest_side_overlay.png`)*
- **Crimson Moss**: `soil_moss_crimson.png` em todas as 6 faces. *(Overlay lateral para solo/rocha: `moss_crimson_side_overlay.png`)*
- **Amber Moss**: `soil_moss_amber.png` em todas as 6 faces. *(Overlay lateral para solo/rocha: `moss_amber_side_overlay.png`)*
- **Cave Moss**: `soil_moss_cave.png` em todas as 6 faces. *(Overlay lateral para solo/rocha: `moss_cave_side_overlay.png`)*

---

### D. Argilas (Clays)

Blocos impermeáveis e homogêneos de sedimentos finos minerais:

- **Gray Clay**: `soil_clay_gray.png` em todas as 6 faces.
- **Red Clay**: `soil_clay_red.png` em todas as 6 faces.
- **White Clay**: `soil_clay_white.png` em todas as 6 faces.
- **Yellow Clay**: `soil_clay_yellow.png` em todas as 6 faces.

---

### E. Areias (Sands)

Blocos homogêneos granulares:

- **Common Sand**: `soil_sand.png` em todas as 6 faces.
- **Red Sand**: `soil_sand_red.png` em todas as 6 faces.
- **Dune Sand**: `soil_sand_dune.png` em todas as 6 faces.
- **White Sand**: `soil_sand_white.png` em todas as 6 faces.
- **Pink Sand**: `soil_sand_pink.png` em todas as 6 faces.
- **Black Sand**: `soil_sand_black.png` em todas as 6 faces.

---

### F. Cascalhos (Gravels - Sistema de Matriz Geológica)

Blocos detríticos compostos por seixos suspensos em diferentes matrizes:

- **Dirty Gravel**: `soil_gravel_dirty.png` em todas as 6 faces (matriz de solo loam).
- **Sandy Gravel**: `soil_gravel_sandy.png` em todas as 6 faces (matriz arenosa fluvial).
- **Ashy Gravel**: `soil_gravel_ashy.png` em todas as 6 faces (matriz de cinza vulcânica escura).
- **Snowy Gravel**: `soil_gravel_snowy.png` em todas as 6 faces (matriz congelada com neve/gelo).

---

### G. Cinzas Vulcânicas (Ashes)

Depósitos piroclásticos homogêneos:

- **Volcanic Ash**: `soil_ash_vulcanic.png` em todas as 6 faces (cinza basáltica escura).
- **Pumice Ash**: `soil_ash_pumice.png` em todas as 6 faces (cinza de pedra-pomes clara).

---

### H. Neves (Snows)

Precipitação sólida e cobertura de criosfera:

- **Snow Block**: `soil_snow.png` em todas as 6 faces (também utilizado como textura do topo de solos e rochas nevados).
- **Powder Snow**: `soil_snow_powder.png` em todas as 6 faces (neve fofa e aerada com física de afundamento).
