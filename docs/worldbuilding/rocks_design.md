# Catálogo e Estrutura de Worldbuilding: Rochas e Geologia

Todas as texturas de rochas ativas estão organizadas na pasta:
📂 **`docs/worldbuilding/rocks`**

Overlays compartilhados de superfície estão na pasta:
📂 **`docs/worldbuilding/overlays`**

---

## 1. Classificação Geológica (Petrológica)

Na geologia do planeta, todas as rochas pertencem a **3 grandes grupos genéticos**, mais a **Camada Primordial Inquebrável do Manto**:

```mermaid
graph TD
    A["Petrologia: Ciclo das Rochas"] --> B["1. Sedimentary (Sedimentares)"]
    A --> C["2. Metamorphic (Metamórficas)"]
    A --> D["3. Igneous (Ígneas / Magmáticas)"]
    A --> E["4. Primordial Mantle (Manto Inquebrável)"]
    
    B --> B1["Clásticas / Detríticas: Argillite, Família Sandstones (6 cores)"]
    B --> B2["Químicas, Silicosas & Evaporitos: Chalk, Dolomite, Karst, Calcite, Chert, Travertine, Limestone, Flint, Alabaster"]
    
    C --> C1["Foliadas: Slate, Gneiss"]
    C --> C2["Não-foliadas / Cristalinas: Marble, Serpentinite, Azurite, Quartzite"]
    
    D --> D1["Extrusivas, Piroclásticas & Vidros: Basalt, Andesite, Tuff, Pumice, Scoria, Obsidian, Pitchstone, Porphyry"]
    D --> D2["Intrusivas (Plutônicas / Pegmatíticas): Granite, Diorite, Gabbro, Peridotite, Cryolite"]
    D --> D3["Minerais Vulcânicos / Hidrotermais & Fusão: Sulfur, Cinnabar, Magma"]
    
    E --> E1["Manto Sólido: rock_mantle (Pressurizado / Frio)"]
    E --> E2["Manto de Pluma: rock_mantle_plume (Plumas Mantélicas / Hotspots)"]
```

---

## 2. Catálogo Atual de Rochas Implementadas

| Rocha | Família Petrológica | Subclasse / Origem | Base Visual / Textura | Papel no Mundo / Biomas Recomendados |
| :--- | :--- | :--- | :--- | :--- |
| **Argillite** | **Sedimentar** | Clástica (folhelho/argilito) | Camadas terracota finas e estratificadas | Cânions argilosos, escarpas fluviais, bacias sedimentares úmidas. |
| **Chalk** | **Sedimentar** | Bioquímica (cocólitos marinhos) | Branco calcário puro e macio | Falésias marinhas oceânicas (estilo Dover), platôs de giz litorâneos. |
| **Dolomite** | **Sedimentar** | Carbonática (dolostone) | Cinza claro alpino resistente (Stone Minecraft) | Cordilheiras colossais e agulhas rochosas (estilo Alpes Dolomitas). |
| **Karst** | **Sedimentar** | Carbonática dissolvida quimicamente | Relevo cinza-ocre poroso erodido | Labirintos cársticos, dolinas, paredões de calcário esculpidos por água. |
| **Calcite** | **Sedimentar / Mineral** | Carbonato cristalino (romboédrico) | Branco perolado translúcido brilhante | Veios hidrotermais, geodos minerais, câmaras de cavernas profundas. |
| **Chert** | **Sedimentar** | Química silicosa (microcristalina) | Camadas quentes caramelo/ocre | Pederneiras, nódulos em leitos calcários, ferramentas pré-históricas. |
| **Travertine** | **Sedimentar** | Carbonática de fontes termais | Faixas fibrosas beges e creme | Terraços termais (Pamukkale/Yellowstone), arquitetura clássica romana. |
| **Limestone** | **Sedimentar** | Carbonática clássica estratificada | Tons creme e cinza-claro com camadas suaves | A rocha sedimentar mais famosa da Terra; cânions fluviais e falésias. |
| **Flint** | **Sedimentar** | Química silicosa (nódulos de sílex escuro) | Cinza-chumbo a preto ceroso vítreo com crosta clara | Nódulos encravados em falésias de giz (Chalk) e calcário, ferramentas pré-históricas. |
| **Alabaster** | **Sedimentar** | Química evaporítica (gipsita maciça) | Branco-marfim suave leitoso com nuvens translúcidas de mel | Bacias evaporíticas de lagos secos, cavernas áridas, arquitetura clássica translúcida. |
| **Azurite** | **Metamórfica / Mineral** | Carbonato básico de cobre | Azul mineral profundo com matriz rochosa | Veios de oxidação de cobre, fendas minerais azuis nobres. |
| **Sandstone** | **Sedimentar** | Clástica (areia comum litificada) | Arenito amarelo-dourado contínuo e sem costura | Paredões de cânions, platôs de deserto, escarpas de areia compactada. |
| **Sandstone Red** | **Sedimentar** | Clástica (areia oxidada com hematita) | Arenito avermelhado/laranja quente | Cânions de arenito vermelho (estilo Monument Valley e Zion, Utah). |
| **Sandstone Dune** | **Sedimentar** | Clástica (areia de duna litificada) | Arenito dourado quente e suave | Dunas fósseis e platôs de transição desértica. |
| **Sandstone White** | **Sedimentar** | Clástica (areia branca de quartzo puro) | Arenito perolado/creme claro | Falésias costeiras tropicais, atóis e praias fósseis. |
| **Sandstone Pink** | **Sedimentar** | Clástica (areia rosa com feldspato) | Arenito rosado / arkose suave | Bacias continentais e leques aluviais de terras áridas. |
| **Sandstone Black** | **Sedimentar** | Clástica (areia vulcânica basáltica) | Arenito cinza-escuro / preto vulcânico | Cânions e escarpas de ilhas vulcânicas de areia negra. |
| **Basalt** | **Ígnea** | Vulcânica extrusiva máfica | Cinza escuro / preto denso (Basalt Minecraft) | Derrames de lava, colunas basálticas, arquipélagos vulcânicos. |
| **Gabbro** | **Ígnea** | Plutônica intrusiva máfica | Cristais grossos pretos e grafite (Blackstone ref) | Raízes profundas da crosta, fundo de fossas abissais, câmaras magmáticas. |
| **Peridotite** | **Ígnea** | Plutônica ultramáfica (manto superior / raízes da crosta) | Verde-oliva escuro denso com cristais vítreos de olivina | Raízes profundas da crosta, assoalho oceânico antigo, transição para o manto. |
| **Cryolite** | **Ígnea** | Intrusiva pegmatítica alcalina (mineral glacial) | Branco-gelo vítreo quase translúcido (aparência de gelo rochoso) | Veios pegmatíticos em regiões árticas/polares, intrusões subglaciais em permafrost. |
| **Granite** | **Ígnea** | Plutônica intrusiva félsica | Matriz rosada com quartzo e mica | Escudos continentais antigos, maciços montanhosos, monólitos continentais. |
| **Diorite** | **Ígnea** | Plutônica intrusiva intermediária | Sal-e-pimenta (plagioclásio + anfibólio) | Zonas de transição magmática, plutons intermediários da crosta média. |
| **Andesite** | **Ígnea** | Vulcânica extrusiva intermediária | Cinza neutro médio homogêneo | Arcos vulcânicos e cinturões orogênicos (como a Cordilheira dos Andes). |
| **Tuff** | **Ígnea** | Vulcânica piroclástica (cinzas consolidadas) | Fragmentos angulares de cinza vulcânica | Encostas de caldeiras vulcânicas, depósitos de explosões piroclásticas. |
| **Pumice** | **Ígnea** | Vulcânica vesicular espumosa | Porosa, cinza-areia ultraleve (End Stone ref) | Encostas de vulcões explosivos, flutua na água, abrasivo natural. |
| **Scoria** | **Ígnea** | Vulcânica vesicular máfica escura | Porosa vulcânica marrom-ferrugem escura (Netherrack ref) | Cones vulcânicos de cinzas e escórias, campos de lava basáltica. |
| **Obsidian** | **Ígnea** | Extrusiva (vidro vulcânico amorfo) | Preto vítreo brilhante com reflexos escuros | Resfriamento ultrarrápido de lava em contato com água/gelo. |
| **Pitchstone** | **Ígnea** | Extrusiva (vidro vulcânico resinoso) | Preto-piche e verde-escuro com brilho graxo/resinoso | Derrames de lava félsica úmida, margens de caldeiras, fendas vulcânicas antigas. |
| **Porphyry** | **Ígnea** | Vulcânica/Subvulcânica com fenocristais | Púrpura imperial com cristais salpicados | A rocha nobre dos imperadores romanos; intrusões magmáticas violetas. |
| **Sulfur** | **Ígnea / Hidrotermal** | Mineral vulcânico nativo | Amarelo-canário vívido puro | Fumarolas ativas, fontes termais sulfurosas e caldeiras vulcânicas. |
| **Cinnabar** | **Ígnea / Hidrotermal** | Minério de sulfeto de mercúrio | Vermelho escarlate brilhante e carmesim | Veios hidrotermais profundos, fontes termais e zonas de fendas magmáticas. |
| **Slate** | **Metamórfica** | Foliada lamelar de baixo grau | Cinza grafite escuro (Deepslate Minecraft) | Encostas íngremes dobradas por tectonismo, lajes e telhas. |
| **Gneiss** | **Metamórfica** | Foliada de alto grau (bandada) | Faixas onduladas cinza-escuro e cinza-médio (Ancient Debris ref) | Escudos continentais antigos e maciços colossais (Pão de Açúcar). |
| **Marble** | **Metamórfica** | Não-foliada (calcário recristalizado) | Branco suave nobre polido (Quartz Minecraft) | Templos clássicos, palácios, monumentos e estátuas. |
| **Serpentinite** | **Metamórfica** | Ultramáfica hidratada do manto | Verde-oliva terroso profundo (Opção C) | Rochas de zonas de subducção oceânica, ornamentação verde natural. |
| **Quartzite** | **Metamórfica** | Não-foliada (arenito recristalizado de alta pressão) | Estratificação terrosa quente e âmbar (estilo pilares de Zhangjiajie) | Pilares colossais, agulhas de montanha, cânions verticais e cristas ultra-resistentes à erosão. |
| **Magma** | **Ígnea / Geotérmica** | Rocha vulcânica em fusão / incandescência | Rocha negra com veios e fendas incandescentes animados | Câmaras magmáticas, leitos de lagos de lava, fontes hidrotermais abissais. |
| **Mantle** | **Manto Primordial** | Rocha mantélica inquebrável | Matriz negra ultra-densa com cristais de olivina | O piso absoluto inquebrável do mundo (*Bedrock*). Sem versão cobbled. |
| **Mantle Plume** | **Manto Primordial** | Rocha mantélica de pluma geotérmica | Matriz negra com veios e pontos de calor incandescente | Hotspots e plumas térmicas na base do mundo. Sem versão cobbled. |

---

## 3. A Camada Primordial do Manto (Bedrock Inquebrável)

Diferente de todas as outras rochas do jogo, o **Manto** é **inquebrável**:
- **`rock_mantle.png`**: Representa a rocha ultra-pressurizada das profundezas da Terra (peridotito / komatiito comprimido a milhões de atmosferas com pequenos cristais de olivina).
- **`rock_mantle_plume.png`**: Representa as plumas mantélicas (*mantle plumes* ou *hotspots*), onde o calor ascendente do núcleo gera pontos e fendas incandescentes em brasa entranhados na rocha sólida (sem ser magma líquido).

Ambos não possuem versão cobbled porque ferramentas de mineração não são capazes de quebrá-los.

---

## 4. Filosofia de Afloramento de Superfície: 12 Rochas de Montanha e Relevo

Rochas que formam relevos externos de montanhas, platôs, falésias e escarpas possuem as variantes de cobertura vegetal (`_grass_side.png`) e climática (`_snow_side.png`):

1. **Argillite**: `rock_argillite_grass_side.png`, `rock_argillite_snow_side.png`
2. **Chalk**: `rock_chalk_grass_side.png`, `rock_chalk_snow_side.png`
3. **Dolomite**: `rock_dolomite_grass_side.png`, `rock_dolomite_snow_side.png`
4. **Slate**: `rock_slate_grass_side.png`, `rock_slate_snow_side.png`
5. **Granite**: `rock_granite_grass_side.png`, `rock_granite_snow_side.png`
6. **Andesite**: `rock_andesite_grass_side.png`, `rock_andesite_snow_side.png`
7. **Basalt**: `rock_basalt_grass_side.png`, `rock_basalt_snow_side.png`
8. **Karst**: `rock_karst_grass_side.png`, `rock_karst_snow_side.png`
9. **Travertine**: `rock_travertine_grass_side.png`, `rock_travertine_snow_side.png`
10. **Gneiss**: `rock_gneiss_grass_side.png`, `rock_gneiss_snow_side.png`
11. **Limestone**: `rock_limestone_grass_side.png`, `rock_limestone_snow_side.png`
12. **Quartzite**: `rock_quartzite_grass_side.png`, `rock_quartzite_snow_side.png`

---

## 5. Catálogo de Texturas em `worldbuilding/rocks/` (Atuais: 102 Texturas)

- **40 Rochas Sólidas**:
  - 38 mineráveis: Argillite, Chalk, Dolomite, Karst, Calcite, Chert, Travertine, Limestone, Flint, Alabaster, Azurite, Sandstone, Sandstone Red, Sandstone Dune, Sandstone White, Sandstone Pink, Sandstone Black, Basalt, Gabbro, Peridotite, Cryolite, Granite, Diorite, Andesite, Tuff, Pumice, Scoria, Obsidian, Pitchstone, Porphyry, Sulfur, Cinnabar, Magma, Slate, Gneiss, Marble, Serpentinite, Quartzite.
  - 2 inquebráveis primordiais: Mantle (`rock_mantle.png`), Mantle Plume (`rock_mantle_plume.png`).
- **38 Rochas Cobbled**: Cobbled de **todas as 38 rochas mineráveis sem exceção** (`cobbled_<nome>.png`).
- **24 Texturas de Afloramento (12 Rochas x Grass & Snow Side)**.

---

## 6. Catálogo Completo de Blocos (102 Blocos)

Este catálogo define a composição de faces (topo, fundo e laterais) e o comportamento básico para cada bloco do sistema geológico do jogo.

---

### A. Rochas Sedimentares (32 Blocos: 16 Sólidos + 16 Cobbled)

Cada uma das 16 rochas sedimentares possui seu bloco de matriz rochosa sólida e sua variante britada (*cobbled*) gerada por mineração:

#### 1. Argillite
- **Argillite**: `rock_argillite.png` em todas as 6 faces.
- **Cobbled Argillite**: `cobbled_argillite.png` em todas as 6 faces.

#### 2. Chalk
- **Chalk**: `rock_chalk.png` em todas as 6 faces.
- **Cobbled Chalk**: `cobbled_chalk.png` em todas as 6 faces.

#### 3. Dolomite
- **Dolomite**: `rock_dolomite.png` em todas as 6 faces.
- **Cobbled Dolomite**: `cobbled_dolomite.png` em todas as 6 faces.

#### 4. Karst
- **Karst**: `rock_karst.png` em todas as 6 faces.
- **Cobbled Karst**: `cobbled_karst.png` em todas as 6 faces.

#### 5. Calcite
- **Calcite**: `rock_calcite.png` em todas as 6 faces.
- **Cobbled Calcite**: `cobbled_calcite.png` em todas as 6 faces.

#### 6. Chert
- **Chert**: `rock_chert.png` em todas as 6 faces.
- **Cobbled Chert**: `cobbled_chert.png` em todas as 6 faces.

#### 7. Travertine
- **Travertine**: `rock_travertine.png` em todas as 6 faces.
- **Cobbled Travertine**: `cobbled_travertine.png` em todas as 6 faces.

#### 8. Limestone
- **Limestone**: `rock_limestone.png` em todas as 6 faces.
- **Cobbled Limestone**: `cobbled_limestone.png` em todas as 6 faces.

#### 9. Flint
- **Flint**: `rock_flint.png` em todas as 6 faces.
- **Cobbled Flint**: `cobbled_flint.png` em todas as 6 faces.

#### 10. Alabaster
- **Alabaster**: `rock_alabaster.png` em todas as 6 faces.
- **Cobbled Alabaster**: `cobbled_alabaster.png` em todas as 6 faces.

#### 11. Sandstone (Arenito Amarelo Comum)
- **Sandstone**: `rock_sandstone.png` em todas as 6 faces.
- **Cobbled Sandstone**: `cobbled_sandstone.png` em todas as 6 faces.

#### 12. Sandstone Red (Arenito Vermelho)
- **Sandstone Red**: `rock_sandstone_red.png` em todas as 6 faces.
- **Cobbled Sandstone Red**: `cobbled_sandstone_red.png` em todas as 6 faces.

#### 13. Sandstone Dune (Arenito Dourado)
- **Sandstone Dune**: `rock_sandstone_dune.png` em todas as 6 faces.
- **Cobbled Sandstone Dune**: `cobbled_sandstone_dune.png` em todas as 6 faces.

#### 14. Sandstone White (Arenito Branco)
- **Sandstone White**: `rock_sandstone_white.png` em todas as 6 faces.
- **Cobbled Sandstone White**: `cobbled_sandstone_white.png` em todas as 6 faces.

#### 15. Sandstone Pink (Arenito Rosa)
- **Sandstone Pink**: `rock_sandstone_pink.png` em todas as 6 faces.
- **Cobbled Sandstone Pink**: `cobbled_sandstone_pink.png` em todas as 6 faces.

#### 16. Sandstone Black (Arenito Negro Vulcânico)
- **Sandstone Black**: `rock_sandstone_black.png` em todas as 6 faces.
- **Cobbled Sandstone Black**: `cobbled_sandstone_black.png` em todas as 6 faces.

---

### B. Rochas Ígneas / Magmáticas (32 Blocos: 16 Sólidos + 16 Cobbled)

#### 1. Basalt
- **Basalt**: `rock_basalt.png` em todas as 6 faces.
- **Cobbled Basalt**: `cobbled_basalt.png` em todas as 6 faces.

#### 2. Gabbro
- **Gabbro**: `rock_gabbro.png` em todas as 6 faces.
- **Cobbled Gabbro**: `cobbled_gabbro.png` em todas as 6 faces.

#### 3. Peridotite
- **Peridotite**: `rock_peridotite.png` em todas as 6 faces.
- **Cobbled Peridotite**: `cobbled_peridotite.png` em todas as 6 faces.

#### 4. Cryolite
- **Cryolite**: `rock_cryolite.png` em todas as 6 faces.
- **Cobbled Cryolite**: `cobbled_cryolite.png` em todas as 6 faces.

#### 5. Granite
- **Granite**: `rock_granite.png` em todas as 6 faces.
- **Cobbled Granite**: `cobbled_granite.png` em todas as 6 faces.

#### 6. Diorite
- **Diorite**: `rock_diorite.png` em todas as 6 faces.
- **Cobbled Diorite**: `cobbled_diorite.png` em todas as 6 faces.

#### 7. Andesite
- **Andesite**: `rock_andesite.png` em todas as 6 faces.
- **Cobbled Andesite**: `cobbled_andesite.png` em todas as 6 faces.

#### 8. Tuff
- **Tuff**: `rock_tuff.png` em todas as 6 faces.
- **Cobbled Tuff**: `cobbled_tuff.png` em todas as 6 faces.

#### 9. Pumice
- **Pumice**: `rock_pumice.png` em todas as 6 faces.
- **Cobbled Pumice**: `cobbled_pumice.png` em todas as 6 faces.

#### 10. Scoria
- **Scoria**: `rock_scoria.png` em todas as 6 faces.
- **Cobbled Scoria**: `cobbled_scoria.png` em todas as 6 faces.

#### 11. Obsidian
- **Obsidian**: `rock_obsidian.png` em todas as 6 faces.
- **Cobbled Obsidian**: `cobbled_obsidian.png` em todas as 6 faces.

#### 12. Pitchstone
- **Pitchstone**: `rock_pitchstone.png` em todas as 6 faces.
- **Cobbled Pitchstone**: `cobbled_pitchstone.png` em todas as 6 faces.

#### 13. Porphyry
- **Porphyry**: `rock_porphyry.png` em todas as 6 faces.
- **Cobbled Porphyry**: `cobbled_porphyry.png` em todas as 6 faces.

#### 14. Sulfur
- **Sulfur**: `rock_sulfur.png` em todas as 6 faces.
- **Cobbled Sulfur**: `cobbled_sulfur.png` em todas as 6 faces.

#### 15. Cinnabar
- **Cinnabar**: `rock_cinnabar.png` em todas as 6 faces.
- **Cobbled Cinnabar**: `cobbled_cinnabar.png` em todas as 6 faces.

#### 16. Magma
- **Magma**: `rock_magma.png` em todas as 6 faces (bloco geotérmico animado; emissão de luz e dano de calor).
- **Cobbled Magma**: `cobbled_magma.png` em todas as 6 faces.

---

### C. Rochas Metamórficas (12 Blocos: 6 Sólidos + 6 Cobbled)

#### 1. Slate
- **Slate**: `rock_slate.png` em todas as 6 faces.
- **Cobbled Slate**: `cobbled_slate.png` em todas as 6 faces.

#### 2. Gneiss
- **Gneiss**: `rock_gneiss.png` em todas as 6 faces.
- **Cobbled Gneiss**: `cobbled_gneiss.png` em todas as 6 faces.

#### 3. Marble
- **Marble**: `rock_marble.png` em todas as 6 faces.
- **Cobbled Marble**: `cobbled_marble.png` em todas as 6 faces.

#### 4. Serpentinite
- **Serpentinite**: `rock_serpentinite.png` em todas as 6 faces.
- **Cobbled Serpentinite**: `cobbled_serpentinite.png` em todas as 6 faces.

#### 5. Azurite
- **Azurite**: `rock_azurite.png` em todas as 6 faces.
- **Cobbled Azurite**: `cobbled_azurite.png` em todas as 6 faces.

#### 6. Quartzite
- **Quartzite**: `rock_quartzite.png` em todas as 6 faces.
- **Cobbled Quartzite**: `cobbled_quartzite.png` em todas as 6 faces.

---

### D. Camada Primordial do Manto (2 Blocos Inquebráveis)

O assoalho tectônico inquebrável (*bedrock*) do planeta. Não geram versão britada:

- **Mantle**: `rock_mantle.png` em todas as 6 faces (manto rígido ultra-pressurizado).
- **Mantle Plume**: `rock_mantle_plume.png` em todas as 6 faces (pluma geotérmica e ponto de calor térmico).

---

### E. Rochas de Afloramento de Superfície (24 Blocos: 12 Grass + 12 Snow)

Blocos geológicos de topo e escarpa que formam os picos, montanhas e falésias do relevo:

#### 1. Rochas com Cobertura de Grama (12 Blocos - Rock Grass)
Para cada bloco:
- Topo: `soil_grass.png` (escala de cinza com biome tint)
- Fundo: `rock_<nome>.png`
- Lateral: `rock_<nome>_grass_side.png` com overlay `rock_grass_side_overlay.png` (em escala de cinza com biome tint na camada de grama).

1. **Argillite Grass**: Topo `soil_grass.png`, fundo `rock_argillite.png`, lateral `rock_argillite_grass_side.png`.
2. **Chalk Grass**: Topo `soil_grass.png`, fundo `rock_chalk.png`, lateral `rock_chalk_grass_side.png`.
3. **Dolomite Grass**: Topo `soil_grass.png`, fundo `rock_dolomite.png`, lateral `rock_dolomite_grass_side.png`.
4. **Slate Grass**: Topo `soil_grass.png`, fundo `rock_slate.png`, lateral `rock_slate_grass_side.png`.
5. **Granite Grass**: Topo `soil_grass.png`, fundo `rock_granite.png`, lateral `rock_granite_grass_side.png`.
6. **Andesite Grass**: Topo `soil_grass.png`, fundo `rock_andesite.png`, lateral `rock_andesite_grass_side.png`.
7. **Basalt Grass**: Topo `soil_grass.png`, fundo `rock_basalt.png`, lateral `rock_basalt_grass_side.png`.
8. **Karst Grass**: Topo `soil_grass.png`, fundo `rock_karst.png`, lateral `rock_karst_grass_side.png`.
9. **Travertine Grass**: Topo `soil_grass.png`, fundo `rock_travertine.png`, lateral `rock_travertine_grass_side.png`.
10. **Gneiss Grass**: Topo `soil_grass.png`, fundo `rock_gneiss.png`, lateral `rock_gneiss_grass_side.png`.
11. **Limestone Grass**: Topo `soil_grass.png`, fundo `rock_limestone.png`, lateral `rock_limestone_grass_side.png`.
12. **Quartzite Grass**: Topo `soil_grass.png`, fundo `rock_quartzite.png`, lateral `rock_quartzite_grass_side.png`.

#### 2. Rochas com Cobertura de Neve (12 Blocos - Rock Snow)
Para cada bloco:
- Topo: `soil_snow.png`
- Fundo: `rock_<nome>.png`
- Lateral: `rock_<nome>_snow_side.png`.

1. **Argillite Snow**: Topo `soil_snow.png`, fundo `rock_argillite.png`, lateral `rock_argillite_snow_side.png`.
2. **Chalk Snow**: Topo `soil_snow.png`, fundo `rock_chalk.png`, lateral `rock_chalk_snow_side.png`.
3. **Dolomite Snow**: Topo `soil_snow.png`, fundo `rock_dolomite.png`, lateral `rock_dolomite_snow_side.png`.
4. **Slate Snow**: Topo `soil_snow.png`, fundo `rock_slate.png`, lateral `rock_slate_snow_side.png`.
5. **Granite Snow**: Topo `soil_snow.png`, fundo `rock_granite.png`, lateral `rock_granite_snow_side.png`.
6. **Andesite Snow**: Topo `soil_snow.png`, fundo `rock_andesite.png`, lateral `rock_andesite_snow_side.png`.
7. **Basalt Snow**: Topo `soil_snow.png`, fundo `rock_basalt.png`, lateral `rock_basalt_snow_side.png`.
8. **Karst Snow**: Topo `soil_snow.png`, fundo `rock_karst.png`, lateral `rock_karst_snow_side.png`.
9. **Travertine Snow**: Topo `soil_snow.png`, fundo `rock_travertine.png`, lateral `rock_travertine_snow_side.png`.
10. **Gneiss Snow**: Topo `soil_snow.png`, fundo `rock_gneiss.png`, lateral `rock_gneiss_snow_side.png`.
11. **Limestone Snow**: Topo `soil_snow.png`, fundo `rock_limestone.png`, lateral `rock_limestone_snow_side.png`.
12. **Quartzite Snow**: Topo `soil_snow.png`, fundo `rock_quartzite.png`, lateral `rock_quartzite_snow_side.png`.

