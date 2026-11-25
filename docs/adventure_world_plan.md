# Adventure Mode World — Design & Implementation Plan (Phase 12)

Documento oficial da arquitetura do **Modo Aventura** (`12.800 × 7.200` blocos em escala **Full** / `6.400 × 3.600` blocos em escala **Medium**, proporção exata **16:9**) e do plano de migração do sistema de biomas (**27 Macro-Biomas + 78 Micro-Biomas + 4 Subterrâneos preservados**) compartilhado entre o **Modo Aventura** e o **Modo Sandbox**.

---

## 1. Camadas Oficiais do Mapa (`assets/maps/adventure/` · `3200 × 1800` · 16:9)

Todas as 6 camadas de dados do Modo Aventura estão finalizadas em resolução nativa **`3200 × 1800`** (1 pixel = `4 × 4` blocos no Full ou `2 × 2` blocos no Medium, perfeitamente alinhadas pixel a pixel):

| Arquivo | Formato | Função no Motor (Rust) |
| :--- | :--- | :--- |
| `assets/maps/adventure/world_landmass.png` | 8-bit Binário (`0` / `255`) | Silhueta exata de terra vs. água (costas, ilhas, rios e lagos) + filtro de navegação no World Map. |
| `assets/maps/adventure/world_continents.png` | RGB (5 cores chapadas) | Identifica Água, Continente Ocidental, Continente Central, Continente Oriental e Ilhas (alimenta o **Filtro de Continentes** no World Map, banners de descoberta e progressão). |
| `assets/maps/adventure/world_regions.png` | RGB (12 regiões oficiais) | Define as 12 grandes regiões geográficas (alimenta o **Filtro de Regiões** no World Map, trilhas sonoras por região e regras de clima). |
| `assets/maps/adventure/heightmap.png` | 16-bit Grayscale (`0..65535`) | Esqueleto 3D do relevo (`Y = -56..180`, nível do mar `Y = 12`), incluindo as 3 trincheiras oceânicas profundas, caldeira do vulcão e cânions escalonados no deserto. |
| `assets/maps/adventure/macro_biomes.png` | RGB (27 cores chapadas) | Mapeia os **27 Macro-Biomas** oficiais (veja `docs/biomes_design.md`). Alimenta o **Filtro de Macro-Biomas** no World Map e a hierarquia principal do Journal. |
| `assets/maps/adventure/micro_biomes.png` | RGB (78 cores chapadas) | Mapeia os **78 Micro-Biomas** oficiais (`100%` travados ao `macro_biomes.png`). Define o solo, subsolo, rocha, árvores, flora, cor da grama/folhagem/água e temperatura/umidade locais. |

### Memória em Runtime (`Arc<AdventureWorldMap>`)

- `heightmap` (`u16`, `3200 × 1800`): **~11,5 MB**
- `macro_biomes` (`u8` ID `1..27`, `3200 × 1800`): **~5,76 MB**
- `micro_biomes` (`u8` ID `1..78`, `3200 × 1800`): **~5,76 MB**
- `regions_and_continents` (`u8` compactado: 4 bits região + 4 bits continente, `3200 × 1800`): **~5,76 MB**
- **Total em RAM**: **~28,8 MB** (compartilhado via `Arc` sem cópias entre todas as threads de geração).

---

## 2. Três Pontos de Spawn Selecionáveis (Continente Ocidental)

Ao iniciar um novo jogo no **Modo Aventura**, o jogador escolhe a escala (**Medium `6.400 × 3.600`** ou **Full `12.800 × 7.200`**) e um dos **3 pontos de partida** no Continente Ocidental (`src/menu/main_menu.rs`):

| # | Região Inicial | UV Normalizado | Coordenadas Full `(X, Z)` | Perfil |
| :-: | :--- | :---: | :---: | :--- |
| **1** | **Northern River Estuary** *(Estuário do Norte)* | `(0.327, 0.294)` | `(-2214, -1483)` | Foz de rio no norte temperado-frio, próximo a florestas boreais e às terras nevadas. |
| **2** | **Central-East Bay** *(Baía Central)* | `(0.410, 0.495)` | `(-1152, -36)` | Baía abrigada cercada por planícies, vales férteis e florestas temperadas — ponto mais equilibrado. |
| **3** | **Southern Coast** *(Costa Sul)* | `(0.303, 0.712)` | `(-2522, +1526)` | Costa sul próxima a bosques, vales, pântano costeiro e às montanhas do sul. |

---

## 3. Nova Arquitetura de Biomas (Compartilhada: Aventura + Sandbox)

Os **40 biomas antigos de superfície** serão substituídos pela nova hierarquia oficial definida em `docs/biomes_design.md` (mantendo intactos os **4 biomas subterrâneos**: `ThermalCaves`, `UndergroundJungle`, `IceCaves`, `MushroomCaves`):

1. **`ContinentId` (5 valores)** — `OceanWater`, `WesternContinent`, `CentralContinent`, `EasternContinent`, `ArchipelagoIslands`.
2. **`WorldRegionId` (12 regiões)** — `OpenOcean`, `CoastalSea`, `SnowyRegion`, `MountainRegion`, `PlainsAndValleys`, `DenseForestRegion`, `DesertRegion`, `ForestRegion`, `AridRegion`, `FrozenRegion`, `RiversAndLakes`, `SpecialRegion`.
3. **`MacroBiome` (27 biomas principais, agrupados em 9 categorias 3×3)** — `CoastalIslands` .. `SnowyWoods`.
4. **`MicroBiome` / `BiomeType` (78 micro-biomas de superfície + 4 subterrâneos = 82 variantes totais)**:
   - Cada `BiomeType` de superfície conhece seu **`parent_macro_biome() -> MacroBiome`** e sua **`category()`**.
   - **No Modo Aventura**: o `MacroBiome` e o `BiomeType` (Micro-Bioma) vêm diretamente de `macro_biomes.png` e `micro_biomes.png` (com leve *domain-warping* de 3–5 blocos nas bordas internas para transições orgânicas ao nível do bloco, sem alterar costas/rios).
   - **No Modo Sandbox**: o `ClimateGenerator` procedural 6D passa a classificar os novos `MacroBiome` e subdividi-los nos 78 `MicroBiome` usando os mesmos parâmetros de temperatura, umidade, continentalidade, erosão e altitude.

---

## 4. Plano de Execução Passo a Passo (Etapas Planejadas)

Para evitar quebrar sistemas existentes, faremos a implementação em **6 etapas modulares**, testando e compilando (`cargo check`) a cada etapa:

### Etapa 1 — Refatoração do Catálogo de Biomas (`src/generation/biome.rs`)

- Criar o enum **`MacroBiome` (27 variantes)** com cores de legenda, chaves de tradução e categorias.
- Atualizar **`BiomeType`** para conter os **78 Micro-Biomas de superfície + 4 Subterrâneos intactos** (removendo os biomas antigos descartados).
- Definir `BiomeConfig` (solo, subsolo, rocha primária, material de penhasco, cor de grama, folhagem, água, temperatura e umidade base) para cada um dos 78 Micro-Biomas reaproveitando os blocos existentes em `src/world/block/`.
- Atualizar `ClimateGenerator` (Sandbox) para emitir os novos biomas.

### Etapa 2 — Adaptação de Árvores, Flora e Estratos (`src/generation/trees.rs`, `flora.rs`, `strata.rs`)

- Mapear cada um dos 78 Micro-Biomas para suas árvores (`Oak`, `Birch`, `Maple`, `Cherry` em `SakuraGrove`, `Pine`, `Yew`, `Redwood`, `Acacia`, `Palm`, `Mahogany`, `Willow`, `Mangrove`, `Charred`, `GiantMushroom`), flora rasteira, cultivos selvagens, corais, *seagrass* (`SeagrassMeadows`) e *kelp* (`KelpForests`).

### Etapa 3 — Loader `3200 × 1800` do Modo Aventura (`src/generation/adventure_map.rs`, `generator.rs`)

- Atualizar `FULL_ADVENTURE_WIDTH` para **`12_800.0`** (`3200 × 4`) e `FULL_ADVENTURE_LENGTH` para **`7_200.0`** (`1800 × 4`).
- Carregar `heightmap.png`, `macro_biomes.png`, `micro_biomes.png`, `world_regions.png` e `world_continents.png` decodificando as cores exatas da tabela oficial de `docs/biomes_design.md`.
- Derivar temperatura e umidade suavemente a partir do Micro-Bioma + latitude + altitude (eliminando a necessidade de um `climate_map.png` separado).

### Etapa 4 — Lagos Elevados e Rios Escalonados (`water_level(x, z)`)

- Calcular no `AdventureWorldMap` a cota local de água `water_level(x, z)` para cada pixel de `River and Lake` (`06`):
  - **Lagos em altitude** (`Calm Lake`) recebem superfície plana `Y_lago` nivelada pela borda do terreno ao redor (cavando apenas 4–6 blocos abaixo de `Y_lago`, sem rasgar até `Y = 12`).
  - **Rios de montanha** (`Rapids`) descem gradualmente da nascente até o mar em degraus naturais e cachoeiras.

### Etapa 5 — Journal, Ícones, Localização (`PT-BR` / `EN-US`) e HUD

- Atualizar `src/journal/` para exibir a nova estrutura em **2 níveis** (**27 Macro-Biomas** expansíveis ou filtráveis mostrando seus **3 Micro-Biomas**), mantendo os 4 subterrâneos.
- Mapear/organizar os ícones de biomas em `assets/textures/interfaces/icons/biomes/` e adicionar todas as chaves em `assets/lang/pt_br.json` e `assets/lang/en_us.json`.

### Etapa 6 — Filtros de Camada no World Map (`M`) & Sistemas Regionais In-Game

- Adicionar seletores de camada no World Map (`src/map/world_map.rs`):
  1. **Terreno Explorado** (padrão)
  2. **Camada: Continentes** (de `world_continents.png`)
  3. **Camada: 12 Regiões** (de `world_regions.png`)
  4. **Camada: 27 Macro-Biomas** (de `macro_biomes.png`)
- Conectar **Banners de Descoberta** ao entrar em novas Regiões/Macro-Biomas e rotear clima/música pelas 12 Regiões Oficiais.
