# Biomes Design — Arquitetura de Macro-Biomas e Micro-Biomas

Organização hierárquica do mundo em **27 Macro-Biomas** (`assets/maps/adventure/macro_biomes.png`) e **78 Micro-Biomas** (`assets/maps/adventure/micro_biomes.png`) em resolução nativa `3200 × 1800` (1 pixel), além dos **4 Biomas Subterrâneos** preservados no subsolo (`ThermalCaves`, `UndergroundJungle`, `IceCaves`, `MushroomCaves`).

---

## 1. Catálogo Oficial, Cores e Estatísticas (27 Macro-Biomas · 78 Micro-Biomas)

### Ilhas *(3 Macro-Biomas · 6 Micro-Biomas independentes)*

> **Regra especial de ilhas**: Os 6 Micro-Biomas insulares são compartilhados pelos 3 Macro-Biomas de ilhas (definidos pela geologia, clima/latitude e porte de cada ilha).

- **`01 Coastal Islands`** · `#5CE0C4` *(385 zonas · 31.339 px · 0,54%)*
- **`02 Oceanic Islands`** · `#1AB2A8` *(156 zonas · 37.890 px · 0,66%)*
- **`03 Continental Islands`** · `#7AD0A8` *(2 zonas · 18.334 px · 0,32%)*
  - `01 Tropical Island` · `#18C49C` *(38 zonas · 10.252 px · 0,18%)*
  - `02 Temperate Island` · `#58D68D` *(13 zonas · 8.922 px · 0,15%)*
  - `03 Tundra Island` · `#C8DCE8` *(48 zonas · 11.144 px · 0,19%)*
  - `04 Frozen Island` · `#5CEEFF` *(313 zonas · 36.450 px · 0,63%)*
  - `05 Rocky Island` · `#8C9BA6` *(37 zonas · 6.006 px · 0,10%)*
  - `06 Coral Island` · `#38E8D8` *(81 zonas · 14.789 px · 0,26%)*

---

### Aquáticos *(3 Macro-Biomas · 9 Micro-Biomas)*

- **`04 Ocean`** · `#143A6E` *(7 zonas · 3.029.247 px · 52,59%)*
  - `07 Warm Ocean` · `#1B4F8C` *(12 zonas · 1.330.688 px · 23,10%)*
  - `08 Cold Ocean` · `#15355E` *(3 zonas · 1.590.187 px · 27,61%)*
  - `09 Ocean Trench` · `#0B1D38` *(3 zonas · 108.372 px · 1,88%)*
- **`05 Coastal Sea`** · `#2E90A8` *(7 zonas · 800.932 px · 13,91%)*
  - `10 Coral Reefs` · `#32B8C8` *(10 zonas · 322.881 px · 5,61%)*
  - `11 Seagrass Meadows` · `#2A969E` *(22 zonas · 89.562 px · 1,55%)*
  - `12 Kelp Forests` · `#1E7282` *(9 zonas · 388.489 px · 6,74%)*
- **`06 River and Lake`** · `#2C84D6` *(24 zonas · 18.906 px · 0,33%)*
  - `13 Rapids` · `#489CE8` *(1 zona · 397 px · 0,01%)*
  - `14 Calm Lake` · `#2474C4` *(20 zonas · 17.010 px · 0,30%)*
  - `15 Estuary` · `#34A4B8` *(6 zonas · 1.499 px · 0,03%)*

---

### Litoral / Praias *(3 Macro-Biomas · 9 Micro-Biomas)*

- **`07 Oceanic Beach`** · `#F4DC98` *(341 zonas · 63.093 px · 1,10%)*
  - `16 Sandy Shores` · `#F6E0A0` *(146 zonas · 42.966 px · 0,75%)*
  - `17 Gravel Beach` · `#C8BE9E` *(101 zonas · 6.572 px · 0,11%)*
  - `18 Dune Beach` · `#EAB86A` *(116 zonas · 13.555 px · 0,24%)*
- **`08 Rocky Coast`** · `#808A94` *(162 zonas · 87.925 px · 1,53%)*
  - `19 Craggy Beach` · `#96A0A8` *(13 zonas · 3.509 px · 0,06%)*
  - `20 Tide Pools` · `#748C94` *(10 zonas · 3.072 px · 0,05%)*
  - `21 Coastal Cliffs` · `#5E666E` *(189 zonas · 81.344 px · 1,41%)*
- **`09 Freshwater Shores`** · `#9AB276` *(115 zonas · 5.329 px · 0,09%)*
  - `22 Mudflats` · `#8E7C5E` *(38 zonas · 3.056 px · 0,05%)*
  - `23 Swampy Shore` · `#688C54` *(56 zonas · 1.647 px · 0,03%)*
  - `24 Reedy Shore` · `#A4BC78` *(21 zonas · 626 px · 0,01%)*

---

### Especiais *(3 Macro-Biomas · 9 Micro-Biomas)*

- **`10 Volcanic Lands`** · `#8A2C24` *(1 zona · 22.358 px · 0,39%)*
  - `25 Caldera` · `#D4582A` *(1 zona · 881 px · 0,02%)*
  - `26 Volcanic Peaks` · `#7A2420` *(1 zona · 9.207 px · 0,16%)*
  - `27 Volcanic Jungle` · `#4A3828` *(1 zona · 12.270 px · 0,21%)*
- **`11 Shadow Lands`** · `#482A5C` *(1 zona · 14.209 px · 0,25%)*
  - `28 Withered Woods` · `#4E3264` *(1 zona · 4.748 px · 0,08%)*
  - `29 Ashen Wastes` · `#382644` *(1 zona · 3.899 px · 0,07%)*
  - `30 Obsidian Mire` · `#241830` *(1 zona · 5.562 px · 0,10%)*
- **`12 Brimstone Basin`** · `#BAAA2A` *(1 zona · 5.137 px · 0,09%)*
  - `31 Acid Pools` · `#C4D428` *(1 zona · 1.531 px · 0,03%)*
  - `32 Sulfur Vents` · `#B89E20` *(1 zona · 2.232 px · 0,04%)*
  - `33 Scorched Flats` · `#8C7224` *(1 zona · 1.374 px · 0,02%)*

---

### Terrestres *(3 Macro-Biomas · 9 Micro-Biomas)*

- **`13 Plains`** · `#84C24E` *(32 zonas · 189.251 px · 3,29%)*
  - `34 Grasslands` · `#84C44E` *(20 zonas · 59.990 px · 1,04%)*
  - `35 Flower Meadow` · `#9ED85C` *(19 zonas · 45.646 px · 0,79%)*
  - `36 Rolling Steppe` · `#A8B852` *(24 zonas · 83.615 px · 1,45%)*
- **`14 Valley`** · `#5CA844` *(19 zonas · 33.966 px · 0,59%)*
  - `37 Fertile Valley` · `#5CA844` *(11 zonas · 24.418 px · 0,42%)*
  - `38 Lavender Valley` · `#9C78B8` *(4 zonas · 4.286 px · 0,07%)*
  - `39 Sakura Grove` · `#E488A8` *(4 zonas · 5.262 px · 0,09%)*
- **`15 Swamp`** · `#4C7044` *(5 zonas · 26.902 px · 0,47%)*
  - `40 Willow Marsh` · `#4E7648` *(4 zonas · 7.830 px · 0,14%)*
  - `41 Mangrove Swamp` · `#3C6244` *(5 zonas · 18.003 px · 0,31%)*
  - `42 Orchid Swamp` · `#648C6E` *(1 zona · 1.069 px · 0,02%)*

---

### Florestais *(3 Macro-Biomas · 9 Micro-Biomas)*

- **`16 Temperate Forest`** · `#38863E` *(12 zonas · 95.758 px · 1,66%)*
  - `43 Verdant Woods` · `#38863E` *(9 zonas · 58.796 px · 1,02%)*
  - `44 Autumn Forest` · `#C47A32` *(7 zonas · 18.447 px · 0,32%)*
  - `45 Misty Woods` · `#2E7454` *(7 zonas · 18.515 px · 0,32%)*
- **`17 Boreal Forest`** · `#266052` *(15 zonas · 75.243 px · 1,31%)*
  - `46 Pine Highlands` · `#2E6E56` *(15 zonas · 59.895 px · 1,04%)*
  - `47 Bleak Taiga` · `#205044` *(6 zonas · 14.030 px · 0,24%)*
  - `48 Shield Forest` · `#3A7868` *(2 zonas · 1.318 px · 0,02%)*
- **`18 Tropical Forest`** · `#165E26` *(9 zonas · 165.665 px · 2,88%)*
  - `49 Dense Rainforest` · `#165E26` *(8 zonas · 145.775 px · 2,53%)*
  - `50 Bamboo Grove` · `#448C36` *(4 zonas · 18.228 px · 0,32%)*
  - `51 Jungle Wetlands` · `#1E7044` *(5 zonas · 1.662 px · 0,03%)*

---

### Montanhosas *(3 Macro-Biomas · 9 Micro-Biomas)*

- **`19 Mountain Range`** · `#9C9CA6` *(16 zonas · 290.405 px · 5,04%)*
  - `52 Alpine Peaks` · `#D4DCE4` *(21 zonas · 114.775 px · 1,99%)*
  - `53 Jagged Peaks` · `#9496A0` *(21 zonas · 19.652 px · 0,34%)*
  - `54 Monolith Cliffs` · `#747682` *(22 zonas · 155.978 px · 2,71%)*
- **`20 Highlands`** · `#827466` *(20 zonas · 29.243 px · 0,51%)*
  - `55 Emerald Foothills` · `#688C5E` *(4 zonas · 4.606 px · 0,08%)*
  - `56 Granite Plateau` · `#8A7C6E` *(3 zonas · 4.058 px · 0,07%)*
  - `57 Misty Crags` · `#6E6A74` *(13 zonas · 20.579 px · 0,36%)*
- **`21 Mountain Gorges`** · `#625670` *(20 zonas · 17.801 px · 0,31%)*
  - `58 Amethyst Canyon` · `#7E5494` *(4 zonas · 1.935 px · 0,03%)*
  - `59 Deep Gorge` · `#585060` *(6 zonas · 4.845 px · 0,08%)*
  - `60 Windy Pass` · `#7A7284` *(10 zonas · 11.021 px · 0,19%)*

---

### Áridas *(3 Macro-Biomas · 9 Micro-Biomas)*

- **`22 Desert`** · `#F2C06C` *(2 zonas · 225.467 px · 3,91%)*
  - `61 Dune Sea` · `#F4C464` *(8 zonas · 65.694 px · 1,14%)*
  - `62 Ancient Sands` · `#E2AE58` *(9 zonas · 157.117 px · 2,73%)*
  - `63 Desert Oasis` · `#38B868` *(7 zonas · 2.656 px · 0,05%)*
- **`23 Savanna`** · `#CC9C48` *(12 zonas · 149.567 px · 2,60%)*
  - `64 Acacia Plains` · `#D0A24C` *(5 zonas · 19.508 px · 0,34%)*
  - `65 Fractured Savanna` · `#B8863C` *(15 zonas · 109.910 px · 1,91%)*
  - `66 Savanna Badlands` · `#A66E34` *(10 zonas · 20.149 px · 0,35%)*
- **`24 Canyons`** · `#C6663A` *(11 zonas · 80.158 px · 1,39%)*
  - `67 Redrock Canyon` · `#C85A32` *(12 zonas · 6.590 px · 0,11%)*
  - `68 Painted Mesas` · `#D87E4A` *(12 zonas · 51.969 px · 0,90%)*
  - `69 Desert Spires` · `#A84A28` *(6 zonas · 21.599 px · 0,37%)*

---

### Gélidas *(3 Macro-Biomas · 9 Micro-Biomas)*

- **`25 Tundra`** · `#E0ECF8` *(18 zonas · 218.349 px · 3,79%)*
  - `70 Snowy Plains` · `#E6F0FA` *(16 zonas · 57.781 px · 1,00%)*
  - `71 Cold Shrubland` · `#B8C8CE` *(10 zonas · 16.844 px · 0,29%)*
  - `72 Permafrost Steppe` · `#CAD8E4` *(14 zonas · 143.724 px · 2,50%)*
- **`26 Glacier`** · `#52ECFC` *(1 zona · 19.977 px · 0,35%)*
  - `73 Glacial Chasm` · `#28B8D8` *(1 zona · 4.900 px · 0,09%)*
  - `74 Ice Sheet` · `#64EEFC` *(3 zonas · 3.682 px · 0,06%)*
  - `75 Frozen Cliffs` · `#42D0E8` *(3 zonas · 11.395 px · 0,20%)*
- **`27 Snowy Woods`** · `#9AC2D6` *(6 zonas · 7.549 px · 0,13%)*
  - `76 Wintry Forest` · `#9CC2D6` *(1 zona · 2.977 px · 0,05%)*
  - `77 Frost Pinewoods` · `#7EA8BE` *(1 zona · 1.131 px · 0,02%)*
  - `78 Icebound Ridge` · `#B4D4E4` *(5 zonas · 3.441 px · 0,06%)*

---

## 2. Resumo Geral dos 27 Macro-Biomas (`macro_biomes.png`)

| # | Cor (Hex) | RGB | Macro-Bioma | Categoria | Zonas | Pixels | % do Mapa | Menor Mancha |
| :-: | :---: | :---: | :--- | :--- | :---: | :---: | :---: | :---: |
| **01** | `#5CE0C4` | `(92, 224, 196)` | `Coastal Islands` | Ilhas | 385 | 31.339 | 0,54% | 1 px |
| **02** | `#1AB2A8` | `(26, 178, 168)` | `Oceanic Islands` | Ilhas | 156 | 37.890 | 0,66% | 1 px |
| **03** | `#7AD0A8` | `(122, 208, 168)` | `Continental Islands` | Ilhas | 2 | 18.334 | 0,32% | 8.829 px |
| **04** | `#143A6E` | `(20, 58, 110)` | `Ocean` | Aquáticos | 7 | 3.029.247 | 52,59% | 1 px |
| **05** | `#2E90A8` | `(46, 144, 168)` | `Coastal Sea` | Aquáticos | 7 | 800.932 | 13,91% | 1 px |
| **06** | `#2C84D6` | `(44, 132, 214)` | `River and Lake` | Aquáticos | 24 | 18.906 | 0,33% | 4 px |
| **07** | `#F4DC98` | `(244, 220, 152)` | `Oceanic Beach` | Litoral | 341 | 63.093 | 1,10% | 1 px |
| **08** | `#808A94` | `(128, 138, 148)` | `Rocky Coast` | Litoral | 162 | 87.925 | 1,53% | 1 px |
| **09** | `#9AB276` | `(154, 178, 118)` | `Freshwater Shores` | Litoral | 115 | 5.329 | 0,09% | 1 px |
| **10** | `#8A2C24` | `(138, 44, 36)` | `Volcanic Lands` | Especiais | 1 | 22.358 | 0,39% | 22.358 px |
| **11** | `#482A5C` | `(72, 42, 92)` | `Shadow Lands` | Especiais | 1 | 14.209 | 0,25% | 14.209 px |
| **12** | `#BAAA2A` | `(186, 170, 42)` | `Brimstone Basin` | Especiais | 1 | 5.137 | 0,09% | 5.137 px |
| **13** | `#84C24E` | `(132, 194, 78)` | `Plains` | Terrestres | 32 | 189.251 | 3,29% | 428 px |
| **14** | `#5CA844` | `(92, 168, 68)` | `Valley` | Terrestres | 19 | 33.966 | 0,59% | 423 px |
| **15** | `#4C7044` | `(76, 112, 68)` | `Swamp` | Terrestres | 5 | 26.902 | 0,47% | 565 px |
| **16** | `#38863E` | `(56, 134, 62)` | `Temperate Forest` | Florestais | 12 | 95.758 | 1,66% | 401 px |
| **17** | `#266052` | `(38, 96, 82)` | `Boreal Forest` | Florestais | 15 | 75.243 | 1,31% | 424 px |
| **18** | `#165E26` | `(22, 94, 38)` | `Tropical Forest` | Florestais | 9 | 165.665 | 2,88% | 423 px |
| **19** | `#9C9CA6` | `(156, 156, 166)` | `Mountain Range` | Montanhosas | 16 | 290.405 | 5,04% | 706 px |
| **20** | `#827466` | `(130, 116, 102)` | `Highlands` | Montanhosas | 20 | 29.243 | 0,51% | 571 px |
| **21** | `#625670` | `(98, 86, 112)` | `Mountain Gorges` | Montanhosas | 20 | 17.801 | 0,31% | 404 px |
| **22** | `#F2C06C` | `(242, 192, 108)` | `Desert` | Áridas | 2 | 225.467 | 3,91% | 49.680 px |
| **23** | `#CC9C48` | `(204, 156, 72)` | `Savanna` | Áridas | 12 | 149.567 | 2,60% | 919 px |
| **24** | `#C6663A` | `(198, 102, 58)` | `Canyons` | Áridas | 11 | 80.158 | 1,39% | 431 px |
| **25** | `#E0ECF8` | `(224, 236, 248)` | `Tundra` | Gélidas | 18 | 218.349 | 3,79% | 480 px |
| **26** | `#52ECFC` | `(82, 236, 252)` | `Glacier` | Gélidas | 1 | 19.977 | 0,35% | 19.977 px |
| **27** | `#9AC2D6` | `(154, 194, 214)` | `Snowy Woods` | Gélidas | 6 | 7.549 | 0,13% | 424 px |

---

## 3. Resumo Geral dos 78 Micro-Biomas (`micro_biomes.png`)

| # | Cor (Hex) | RGB | Micro-Bioma | Macro-Bioma Pai | Zonas | Pixels | % do Mapa | Menor Mancha |
| :-: | :---: | :---: | :--- | :--- | :---: | :---: | :---: | :---: |
| **01** | `#18C49C` | `(24, 196, 156)` | `Tropical Island` | `01–03 Islands` | 38 | 10.252 | 0,18% | 1 px |
| **02** | `#58D68D` | `(88, 214, 141)` | `Temperate Island` | `01–03 Islands` | 13 | 8.922 | 0,15% | 6 px |
| **03** | `#C8DCE8` | `(200, 220, 232)` | `Tundra Island` | `01–03 Islands` | 48 | 11.144 | 0,19% | 2 px |
| **04** | `#5CEEFF` | `(92, 238, 255)` | `Frozen Island` | `01–03 Islands` | 313 | 36.450 | 0,63% | 1 px |
| **05** | `#8C9BA6` | `(140, 155, 166)` | `Rocky Island` | `01–03 Islands` | 37 | 6.006 | 0,10% | 2 px |
| **06** | `#38E8D8` | `(56, 232, 216)` | `Coral Island` | `01–03 Islands` | 81 | 14.789 | 0,26% | 1 px |
| **07** | `#1B4F8C` | `(27, 79, 140)` | `Warm Ocean` | `04 Ocean` | 12 | 1.330.688 | 23,10% | 1 px |
| **08** | `#15355E` | `(21, 53, 94)` | `Cold Ocean` | `04 Ocean` | 3 | 1.590.187 | 27,61% | 4.818 px |
| **09** | `#0B1D38` | `(11, 29, 56)` | `Ocean Trench` | `04 Ocean` | 3 | 108.372 | 1,88% | 7.731 px |
| **10** | `#32B8C8` | `(50, 184, 200)` | `Coral Reefs` | `05 Coastal Sea` | 10 | 322.881 | 5,61% | 201 px |
| **11** | `#2A969E` | `(42, 150, 158)` | `Seagrass Meadows` | `05 Coastal Sea` | 22 | 89.562 | 1,55% | 534 px |
| **12** | `#1E7282` | `(30, 114, 130)` | `Kelp Forests` | `05 Coastal Sea` | 9 | 388.489 | 6,74% | 1 px |
| **13** | `#489CE8` | `(72, 156, 232)` | `Rapids` | `06 River and Lake` | 1 | 397 | 0,01% | 397 px |
| **14** | `#2474C4` | `(36, 116, 196)` | `Calm Lake` | `06 River and Lake` | 20 | 17.010 | 0,30% | 11 px |
| **15** | `#34A4B8` | `(52, 164, 184)` | `Estuary` | `06 River and Lake` | 6 | 1.499 | 0,03% | 1 px |
| **16** | `#F6E0A0` | `(246, 224, 160)` | `Sandy Shores` | `07 Oceanic Beach` | 146 | 42.966 | 0,75% | 1 px |
| **17** | `#C8BE9E` | `(200, 190, 158)` | `Gravel Beach` | `07 Oceanic Beach` | 101 | 6.572 | 0,11% | 1 px |
| **18** | `#EAB86A` | `(234, 184, 106)` | `Dune Beach` | `07 Oceanic Beach` | 116 | 13.555 | 0,24% | 1 px |
| **19** | `#96A0A8` | `(150, 160, 168)` | `Craggy Beach` | `08 Rocky Coast` | 13 | 3.509 | 0,06% | 1 px |
| **20** | `#748C94` | `(116, 140, 148)` | `Tide Pools` | `08 Rocky Coast` | 10 | 3.072 | 0,05% | 10 px |
| **21** | `#5E666E` | `(94, 102, 110)` | `Coastal Cliffs` | `08 Rocky Coast` | 189 | 81.344 | 1,41% | 1 px |
| **22** | `#8E7C5E` | `(142, 124, 94)` | `Mudflats` | `09 Freshwater Shores` | 38 | 3.056 | 0,05% | 1 px |
| **23** | `#688C54` | `(104, 140, 84)` | `Swampy Shore` | `09 Freshwater Shores` | 56 | 1.647 | 0,03% | 1 px |
| **24** | `#A4BC78` | `(164, 188, 120)` | `Reedy Shore` | `09 Freshwater Shores` | 21 | 626 | 0,01% | 1 px |
| **25** | `#D4582A` | `(212, 88, 42)` | `Caldera` | `10 Volcanic Lands` | 1 | 881 | 0,02% | 881 px |
| **26** | `#7A2420` | `(122, 36, 32)` | `Volcanic Peaks` | `10 Volcanic Lands` | 1 | 9.207 | 0,16% | 9.207 px |
| **27** | `#4A3828` | `(74, 56, 40)` | `Volcanic Jungle` | `10 Volcanic Lands` | 1 | 12.270 | 0,21% | 12.270 px |
| **28** | `#4E3264` | `(78, 50, 100)` | `Withered Woods` | `11 Shadow Lands` | 1 | 4.748 | 0,08% | 4.748 px |
| **29** | `#382644` | `(56, 38, 68)` | `Ashen Wastes` | `11 Shadow Lands` | 1 | 3.899 | 0,07% | 3.899 px |
| **30** | `#241830` | `(36, 24, 48)` | `Obsidian Mire` | `11 Shadow Lands` | 1 | 5.562 | 0,10% | 5.562 px |
| **31** | `#C4D428` | `(196, 212, 40)` | `Acid Pools` | `12 Brimstone Basin` | 1 | 1.531 | 0,03% | 1.531 px |
| **32** | `#B89E20` | `(184, 158, 32)` | `Sulfur Vents` | `12 Brimstone Basin` | 1 | 2.232 | 0,04% | 2.232 px |
| **33** | `#8C7224` | `(140, 114, 36)` | `Scorched Flats` | `12 Brimstone Basin` | 1 | 1.374 | 0,02% | 1.374 px |
| **34** | `#84C44E` | `(132, 196, 78)` | `Grasslands` | `13 Plains` | 20 | 59.990 | 1,04% | 254 px |
| **35** | `#9ED85C` | `(158, 216, 92)` | `Flower Meadow` | `13 Plains` | 19 | 45.646 | 0,79% | 187 px |
| **36** | `#A8B852` | `(168, 184, 82)` | `Rolling Steppe` | `13 Plains` | 24 | 83.615 | 1,45% | 189 px |
| **37** | `#5CA844` | `(92, 168, 68)` | `Fertile Valley` | `14 Valley` | 11 | 24.418 | 0,42% | 423 px |
| **38** | `#9C78B8` | `(156, 120, 184)` | `Lavender Valley` | `14 Valley` | 4 | 4.286 | 0,07% | 533 px |
| **39** | `#E488A8` | `(228, 136, 168)` | `Sakura Grove` | `14 Valley` | 4 | 5.262 | 0,09% | 854 px |
| **40** | `#4E7648` | `(78, 118, 72)` | `Willow Marsh` | `15 Swamp` | 4 | 7.830 | 0,14% | 264 px |
| **41** | `#3C6244` | `(60, 98, 68)` | `Mangrove Swamp` | `15 Swamp` | 5 | 18.003 | 0,31% | 565 px |
| **42** | `#648C6E` | `(100, 140, 110)` | `Orchid Swamp` | `15 Swamp` | 1 | 1.069 | 0,02% | 1.069 px |
| **43** | `#38863E` | `(56, 134, 62)` | `Verdant Woods` | `16 Temperate Forest` | 9 | 58.796 | 1,02% | 326 px |
| **44** | `#C47A32` | `(196, 122, 50)` | `Autumn Forest` | `16 Temperate Forest` | 7 | 18.447 | 0,32% | 401 px |
| **45** | `#2E7454` | `(46, 116, 84)` | `Misty Woods` | `16 Temperate Forest` | 7 | 18.515 | 0,32% | 992 px |
| **46** | `#2E6E56` | `(46, 110, 86)` | `Pine Highlands` | `17 Boreal Forest` | 15 | 59.895 | 1,04% | 424 px |
| **47** | `#205044` | `(32, 80, 68)` | `Bleak Taiga` | `17 Boreal Forest` | 6 | 14.030 | 0,24% | 506 px |
| **48** | `#3A7868` | `(58, 120, 104)` | `Shield Forest` | `17 Boreal Forest` | 2 | 1.318 | 0,02% | 245 px |
| **49** | `#165E26` | `(22, 94, 38)` | `Dense Rainforest` | `18 Tropical Forest` | 8 | 145.775 | 2,53% | 593 px |
| **50** | `#448C36` | `(68, 140, 54)` | `Bamboo Grove` | `18 Tropical Forest` | 4 | 18.228 | 0,32% | 1.383 px |
| **51** | `#1E7044` | `(30, 112, 68)` | `Jungle Wetlands` | `18 Tropical Forest` | 5 | 1.662 | 0,03% | 264 px |
| **52** | `#D4DCE4` | `(212, 220, 228)` | `Alpine Peaks` | `19 Mountain Range` | 21 | 114.775 | 1,99% | 261 px |
| **53** | `#9496A0` | `(148, 150, 160)` | `Jagged Peaks` | `19 Mountain Range` | 21 | 19.652 | 0,34% | 199 px |
| **54** | `#747682` | `(116, 118, 130)` | `Monolith Cliffs` | `19 Mountain Range` | 22 | 155.978 | 2,71% | 185 px |
| **55** | `#688C5E` | `(104, 140, 94)` | `Emerald Foothills` | `20 Highlands` | 4 | 4.606 | 0,08% | 571 px |
| **56** | `#8A7C6E` | `(138, 124, 110)` | `Granite Plateau` | `20 Highlands` | 3 | 4.058 | 0,07% | 726 px |
| **57** | `#6E6A74` | `(110, 106, 116)` | `Misty Crags` | `20 Highlands` | 13 | 20.579 | 0,36% | 707 px |
| **58** | `#7E5494` | `(126, 84, 148)` | `Amethyst Canyon` | `21 Mountain Gorges` | 4 | 1.935 | 0,03% | 423 px |
| **59** | `#585060` | `(88, 80, 96)` | `Deep Gorge` | `21 Mountain Gorges` | 6 | 4.845 | 0,08% | 404 px |
| **60** | `#7A7284` | `(122, 114, 132)` | `Windy Pass` | `21 Mountain Gorges` | 10 | 11.021 | 0,19% | 501 px |
| **61** | `#F4C464` | `(244, 196, 100)` | `Dune Sea` | `22 Desert` | 8 | 65.694 | 1,14% | 336 px |
| **62** | `#E2AE58` | `(226, 174, 88)` | `Ancient Sands` | `22 Desert` | 9 | 157.117 | 2,73% | 210 px |
| **63** | `#38B868` | `(56, 184, 104)` | `Desert Oasis` | `22 Desert` | 7 | 2.656 | 0,05% | 1 px |
| **64** | `#D0A24C` | `(208, 162, 76)` | `Acacia Plains` | `23 Savanna` | 5 | 19.508 | 0,34% | 223 px |
| **65** | `#B8863C` | `(184, 134, 60)` | `Fractured Savanna` | `23 Savanna` | 15 | 109.910 | 1,91% | 217 px |
| **66** | `#A66E34` | `(166, 110, 52)` | `Savanna Badlands` | `23 Savanna` | 10 | 20.149 | 0,35% | 243 px |
| **67** | `#C85A32` | `(200, 90, 50)` | `Redrock Canyon` | `24 Canyons` | 12 | 6.590 | 0,11% | 232 px |
| **68** | `#D87E4A` | `(216, 126, 74)` | `Painted Mesas` | `24 Canyons` | 12 | 51.969 | 0,90% | 533 px |
| **69** | `#A84A28` | `(168, 74, 40)` | `Desert Spires` | `24 Canyons` | 6 | 21.599 | 0,37% | 1.069 px |
| **70** | `#E6F0FA` | `(230, 240, 250)` | `Snowy Plains` | `25 Tundra` | 16 | 57.781 | 1,00% | 369 px |
| **71** | `#B8C8CE` | `(184, 200, 206)` | `Cold Shrubland` | `25 Tundra` | 10 | 16.844 | 0,29% | 301 px |
| **72** | `#CAD8E4` | `(202, 216, 228)` | `Permafrost Steppe` | `25 Tundra` | 14 | 143.724 | 2,50% | 550 px |
| **73** | `#28B8D8` | `(40, 184, 216)` | `Glacial Chasm` | `26 Glacier` | 1 | 4.900 | 0,09% | 4.900 px |
| **74** | `#64EEFC` | `(100, 238, 252)` | `Ice Sheet` | `26 Glacier` | 3 | 3.682 | 0,06% | 192 px |
| **75** | `#42D0E8` | `(66, 208, 232)` | `Frozen Cliffs` | `26 Glacier` | 3 | 11.395 | 0,20% | 346 px |
| **76** | `#9CC2D6` | `(156, 194, 214)` | `Wintry Forest` | `27 Snowy Woods` | 1 | 2.977 | 0,05% | 2.977 px |
| **77** | `#7EA8BE` | `(126, 168, 190)` | `Frost Pinewoods` | `27 Snowy Woods` | 1 | 1.131 | 0,02% | 1.131 px |
| **78** | `#B4D4E4` | `(180, 212, 228)` | `Icebound Ridge` | `27 Snowy Woods` | 5 | 3.441 | 0,06% | 424 px |

---

## 4. Regiões Oficiais do Mundo (`assets/maps/adventure/world_regions.png`)

| Código | Cor (Hex) | Região Oficial | Categorias de Macro-Biomas Relacionadas |
| :---: | :---: | :--- | :--- |
| **01** | `#184880` | **Mar Aberto** | Aquáticos (`Ocean`) |
| **02** | `#369AAE` | **Regiões Costeiras** | Aquáticos (`Coastal Sea`), Litoral (`Oceanic Beach`, `Rocky Coast`), Ilhas (`Coastal Islands`) |
| **03** | `#E4EEF8` | **Regiões Nevadas** | Gélidas (`Tundra`, `Snowy Woods`), Ilhas (`Tundra Island`) |
| **04** | `#8A888E` | **Regiões Montanhosas** | Montanhosas (`Mountain Range`, `Highlands`, `Mountain Gorges`), Áridas (`Canyons` no deserto) |
| **05** | `#72A848` | **Planícies e Vales** | Terrestres (`Plains`, `Valley`, `Swamp`) |
| **06** | `#186028` | **Regiões Florestais Densas** | Florestais (`Tropical Forest`), Terrestres (`Swamp`), Ilhas (`Tropical Island`) |
| **07** | `#ECBA72` | **Regiões Desérticas** | Áridas (`Desert`) |
| **08** | `#347C3A` | **Regiões Florestais** | Florestais (`Temperate Forest`, `Boreal Forest`), Ilhas (`Temperate Island`) |
| **09** | `#C29C52` | **Regiões Áridas** | Áridas (`Savanna`, bordas de `Canyons`) |
| **10** | `#49FAFF` | **Regiões Geladas** | Gélidas (`Glacier`), Ilhas (`Frozen Island`) |
| **11** | `#308AC6` | **Rios e Lagos** | Aquáticos (`River and Lake`), Litoral (`Freshwater Shores`) |
| **12** | `#463A3C` / `#B8712D` | **Regiões Especiais** | Especiais (`Volcanic Lands`, `Shadow Lands`, `Brimstone Basin`) |
