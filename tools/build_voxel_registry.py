import os

with open('all_textures.txt', 'r', encoding='utf-8-sig') as f:
    textures_on_disk = set(line.strip().replace('.png', '') for line in f if line.strip())

# The 24 rocky block types:
rock_types = [
    ("Andesite", "andesite"),
    ("Azurite", "azurite"),
    ("Basalt", "basalt"),
    ("Black Sandstone", "black_sandstone"),
    ("Calcite", "calcite"),
    ("Chalk", "chalk"),
    ("Chert", "chert"),
    ("Cinnabar", "cinnabar"),
    ("Diorite", "diorite"),
    ("Gabbro", "gabbro"),
    ("Granite", "granite"),
    ("Karst", "karst"),
    ("Limestone", "limestone"),
    ("Marl", "marl"),
    ("Pitchstone", "pitchstone"),
    ("Porphyry", "porphyry"),
    ("Red Sandstone", "red_sandstone"),
    ("Sandstone", "sandstone"),
    ("Scoria", "scoria"),
    ("Serpentine", "serpentine"),
    ("Slate", "slate"),
    ("Stone", "stone"),
    ("Tuffite", "tuffite"),
    ("White Sandstone", "white_sandstone"),
]

# The 8 special rocks:
special_rocks = [
    ("Rock_Alabaster", "rock_alabaster", "Alabaster", [220, 220, 220, 255]),
    ("Rock_Brimstone", "rock_brimstone", "Brimstone", [210, 180, 50, 255]),
    ("Rock_Cryolite", "rock_cryolite", "Cryolite", [200, 230, 240, 255]),
    ("Rock_Dreadstone", "rock_dreadstone", "Dreadstone", [35, 30, 40, 255]),
    ("Rock_Flint", "rock_flint", "Flint", [55, 55, 60, 255]),
    ("Rock_Magma", "rock_magma", "Magma", [180, 70, 30, 255]),
    ("Rock_Obsidian", "rock_obsidian", "Obsidian", [25, 20, 35, 255]),
    ("Rock_Terracotta", "rock_terracotta", "Terracotta", [165, 95, 65, 255]),
]

# Soil blocks (32):
soil_blocks = [
    ("Soil_Ash", "soil_ash", "Ash", [140, 140, 145, 255]),
    ("Soil_Black_Sand", "soil_black_sand", "Black Sand", [45, 45, 50, 255]),
    ("Soil_Clay", "soil_clay", "Clay", [150, 155, 175, 255]),
    ("Soil_Dirt", "soil_dirt", "Dirt", [107, 66, 33, 255]),
    ("Soil_Grass", "soil_grass", "Grass", [110, 180, 80, 255]),
    ("Soil_Gravel", "soil_gravel", "Gravel", [130, 125, 125, 255]),
    ("Soil_Moss", "soil_moss", "Moss", [80, 130, 60, 255]),
    ("Soil_Mud", "soil_mud", "Mud", [85, 60, 45, 255]),
    ("Soil_Mulch", "soil_mulch", "Mulch", [90, 55, 35, 255]),
    ("Soil_Packed_Dirt", "soil_packed_dirt", "Packed Dirt", [100, 60, 30, 255]),
    ("Soil_Packed_Mud", "soil_packed_mud", "Packed Mud", [80, 55, 40, 255]),
    ("Soil_Packed_Peat", "soil_packed_peat", "Packed Peat", [70, 50, 35, 255]),
    ("Soil_Packed_Silt", "soil_packed_silt", "Packed Silt", [115, 110, 100, 255]),
    ("Soil_Peat", "soil_peat", "Peat", [75, 55, 40, 255]),
    ("Soil_Peat_Grass", "soil_grass", "Peat Grass", [95, 150, 60, 255]),
    ("Soil_Peat_Mulch", "soil_peat_mulch", "Peat Mulch", [80, 50, 35, 255]),
    ("Soil_Red_Moss", "soil_red_moss", "Red Moss", [175, 45, 45, 255]),
    ("Soil_Red_Sand", "soil_red_sand", "Red Sand", [190, 100, 50, 255]),
    ("Soil_Rooted_Dirt", "soil_rooted_dirt", "Rooted Dirt", [115, 75, 40, 255]),
    ("Soil_Sand", "soil_sand", "Sand", [209, 194, 128, 255]),
    ("Soil_Scorched_Black_Sand", "soil_scorched_black_sand", "Scorched Black Sand", [35, 35, 40, 255]),
    ("Soil_Scorched_Red_Sand", "soil_scorched_red_sand", "Scorched Red Sand", [160, 70, 40, 255]),
    ("Soil_Scorched_Sand", "soil_scorched_sand", "Scorched Sand", [175, 150, 100, 255]),
    ("Soil_Scorched_White_Sand", "soil_scorched_white_sand", "Scorched White Sand", [190, 190, 185, 255]),
    ("Soil_Silt", "soil_silt", "Silt", [130, 125, 115, 255]),
    ("Soil_Silt_Grass", "soil_grass", "Silt Grass", [110, 160, 70, 255]),
    ("Soil_Silt_Mulch", "soil_silt_mulch", "Silt Mulch", [100, 80, 60, 255]),
    ("Soil_Snow", "soil_snow", "Snow", [240, 245, 255, 255]),
    ("Soil_Snowy_Grass", "soil_snow", "Snowy Grass", [240, 245, 255, 255]),
    ("Soil_Snowy_Peat", "soil_snow", "Snowy Peat", [220, 225, 235, 255]),
    ("Soil_Snowy_Silt", "soil_snow", "Snowy Silt", [225, 230, 240, 255]),
    ("Soil_White_Sand", "soil_white_sand", "White Sand", [235, 230, 215, 255]),
]

# Liquid blocks (8):
liquid_blocks = [
    ("Liquid_Acid", "liquid_acid_still", "Acid", [120, 230, 40, 255]),
    ("Liquid_Blood", "liquid_blood_still", "Blood", [180, 20, 20, 255]),
    ("Liquid_Lava", "liquid_lava_still", "Lava", [230, 100, 20, 255]),
    ("Liquid_Molten", "liquid_molten_still", "Molten", [255, 140, 0, 255]),
    ("Liquid_Ooze", "liquid_ooze_still", "Ooze", [130, 50, 160, 255]),
    ("Liquid_Sludge", "liquid_sludge_still", "Sludge", [70, 80, 50, 255]),
    ("Liquid_Tar", "liquid_tar_still", "Tar", [25, 25, 30, 255]),
    ("Liquid_Water", "liquid_water_still", "Water", [60, 140, 220, 255]),
]

# Frost blocks (4):
frost_blocks = [
    ("Frost_Black_Ice", "frost_black_ice", "Black Ice", [40, 50, 70, 255]),
    ("Frost_Fragile_Ice", "frost_fragile_ice", "Fragile Ice", [140, 185, 235, 200]),
    ("Frost_Ice", "frost_ice", "Ice", [140, 185, 235, 220]),
    ("Frost_Packed_Ice", "frost_packed_ice", "Packed Ice", [160, 200, 245, 255]),
]

# Testing/Debug blocks (8):
testing_blocks = [
    ("Null_Block", "null_block", "Null Block", [255, 0, 255, 255]),
    ("Null_Liquid", "null_liquid", "Null Liquid", [255, 0, 255, 255]),
    ("Test_Accept", "test_accept", "Test Accept", [50, 200, 50, 255]),
    ("Test_Debug", "test_debug", "Test Debug", [200, 200, 50, 255]),
    ("Test_Numbers", "test_numbers", "Test Numbers", [200, 200, 50, 255]),
    ("Test_Fail", "test_fail", "Test Fail", [200, 50, 50, 255]),
    ("Test_Instance", "test_instance", "Test Instance", [200, 200, 50, 255]),
    ("Test_Start", "test_start", "Test Start", [50, 150, 250, 255]),
]

# Light source blocks (8):
light_blocks = [
    ("Emit_Blue_Light", "emit_blue_light", "Blue Light Block", [64, 128, 255, 255]),
    ("Emit_Blue_Torch", "emit_blue_torch", "Blue Torch", [64, 128, 255, 255]),
    ("Emit_Cold_Light", "emit_cold_light", "Cold Light Block", [200, 230, 255, 255]),
    ("Emit_Green_Light", "emit_green_light", "Green Light Block", [64, 255, 64, 255]),
    ("Emit_Green_Torch", "emit_green_torch", "Green Torch", [64, 255, 64, 255]),
    ("Emit_Red_Light", "emit_red_light", "Red Light Block", [255, 64, 64, 255]),
    ("Emit_Red_Torch", "emit_red_torch", "Red Torch", [255, 64, 64, 255]),
    ("Emit_Warm_Light", "emit_warm_light", "Warm Light Block", [255, 199, 64, 255]),
]

# Decoration blocks (16):
deco_blocks = [
    ("Deco_Barrel", "deco_barrel", "Barrel", [133, 94, 56, 255]),
    ("Deco_Basket", "deco_basket_side", "Basket", [170, 130, 75, 255]),
    ("Deco_Bone", "deco_bone", "Bone Block", [225, 220, 200, 255]),
    ("Deco_Bookshelf", "deco_bookshelf", "Bookshelf", [140, 100, 60, 255]),
    ("Deco_Brick", "deco_bricks", "Brick", [160, 80, 60, 255]),
    ("Deco_Fabric", "deco_fabric", "Fabric", [210, 210, 210, 255]),
    ("Deco_Flesh", "deco_flesh", "Flesh", [160, 60, 60, 255]),
    ("Deco_Glass", "deco_glass", "Glass", [200, 225, 235, 120]),
    ("Deco_Hay", "deco_hay", "Hay", [200, 180, 70, 255]),
    ("Deco_Plaster", "deco_plaster", "Plaster", [215, 215, 210, 255]),
    ("Deco_Slime", "deco_slime", "Slime", [100, 210, 90, 220]),
    ("Deco_Stone_Path", "deco_stone_path", "Stone Path", [120, 120, 120, 255]),
    ("Deco_Thatch", "deco_thatch", "Thatch", [190, 160, 80, 255]),
    ("Deco_Wax", "deco_wax", "Wax", [230, 210, 120, 255]),
    ("Deco_Wicker", "deco_wicker", "Wicker", [180, 140, 80, 255]),
    ("Deco_Wool", "deco_wool", "Wool", [230, 230, 230, 255]),
]

# Wood blocks (48):
wood_trees = [
    ("Acacia", "acacia", [160, 85, 45, 255], [180, 100, 50, 255], [100, 140, 40, 255]),
    ("Birch", "birch", [225, 222, 210, 255], [200, 195, 180, 255], [133, 199, 56, 255]),
    ("Mahogany", "mahogany", [100, 50, 40, 255], [120, 60, 45, 255], [70, 120, 45, 255]),
    ("Mangrove", "mangrove", [110, 75, 60, 255], [130, 85, 65, 255], [65, 125, 55, 255]),
    ("Maple", "maple", [130, 90, 60, 255], [150, 105, 70, 255], [200, 80, 40, 255]),
    ("Oak", "oak", [133, 94, 56, 255], [155, 115, 75, 255], [87, 166, 46, 255]),
    ("Palm", "palm", [140, 110, 70, 255], [160, 130, 85, 255], [90, 160, 50, 255]),
    ("Pine", "pine", [74, 48, 28, 255], [115, 80, 50, 255], [46, 107, 66, 255]),
    ("Willow", "willow", [90, 85, 60, 255], [110, 105, 75, 255], [75, 130, 50, 255]),
    ("Yew", "yew", [105, 65, 45, 255], [125, 80, 55, 255], [45, 95, 55, 255]),
]

wood_blocks = []
for tname, tslug, bark_col, plank_col, leaf_col in wood_trees:
    wood_blocks.append((f"Tree_{tname}_Bark", f"tree_{tslug}_bark", f"{tname} Bark", bark_col))
    wood_blocks.append((f"Tree_{tname}_Log", f"tree_{tslug}_log", f"{tname} Log", bark_col))
    if tslug == "maple":
        wood_blocks.append(("Tree_Maple_Leaves_Red", "tree_maple_leaves_red", "Red Maple Leaves", [185, 45, 30, 255]))
        wood_blocks.append(("Tree_Maple_Leaves_Orange", "tree_maple_leaves_orange_", "Orange Maple Leaves", [225, 115, 25, 255]))
        wood_blocks.append(("Tree_Maple_Leaves_Yellow", "tree_maple_leaves_yellow", "Yellow Maple Leaves", [225, 185, 30, 255]))
    else:
        wood_blocks.append((f"Tree_{tname}_Leaves", f"tree_{tslug}_leaves", f"{tname} Leaves", leaf_col))
    wood_blocks.append((f"Tree_{tname}_Planks", f"tree_{tslug}_planks", f"{tname} Planks", plank_col))
    if tname == "Mangrove":
        wood_blocks.append(("Tree_Mangrove_Roots", "tree_mangrove_roots", "Mangrove Roots", [95, 65, 50, 255]))

wood_blocks.append(("Tree_Cactus", "tree_cactus_side", "Cactus", [85, 135, 45, 255]))

# Charred wood
wood_blocks.append(("Tree_Charred_Bark", "tree_charred_bark", "Charred Wood Bark", [30, 30, 30, 255]))
wood_blocks.append(("Tree_Charred_Log", "tree_charred_log", "Charred Wood Log", [35, 35, 35, 255]))
wood_blocks.append(("Tree_Charred_Planks", "tree_charred_planks", "Charred Wood Planks", [45, 45, 45, 255]))

# Dead wood
wood_blocks.append(("Tree_Dead_Bark", "tree_dead_bark", "Dead Wood Bark", [110, 100, 90, 255]))
wood_blocks.append(("Tree_Dead_Log", "tree_dead_log", "Dead Wood Log", [120, 110, 100, 255]))
wood_blocks.append(("Tree_Dead_Planks", "tree_dead_planks", "Dead Wood Planks", [135, 125, 115, 255]))

# Aquatic blocks (8):
aqua_blocks = [
    ("Aqua_Algae_Mat", "aqua_algae_mat", "Algae Mat", [50, 120, 70, 255]),
    ("Aqua_Brain_Coral", "aqua_brain_coral", "Brain Coral", [220, 100, 130, 255]),
    ("Aqua_Bubble_Coral", "aqua_bubble_coral", "Bubble Coral", [180, 70, 160, 255]),
    ("Aqua_Fire_Coral", "aqua_fire_coral", "Fire Coral", [220, 50, 50, 255]),
    ("Aqua_Geothermal_Vent", "aqua_geothermal_vent", "Geothermal Vent", [80, 80, 80, 255]),
    ("Aqua_Horn_Coral", "aqua_horn_coral", "Horn Coral", [220, 180, 60, 255]),
    ("Aqua_Sponge", "aqua_sponge", "Sponge", [190, 175, 50, 255]),
    ("Aqua_Tube_Coral", "aqua_tube_coral", "Tube Coral", [60, 120, 220, 255]),
]

# Generate Rocky blocks list (96)
rock_colors = {
    "andesite": [136, 136, 136, 255],
    "azurite": [45, 85, 160, 255],
    "basalt": [75, 75, 80, 255],
    "black_sandstone": [50, 50, 55, 255],
    "calcite": [220, 220, 225, 255],
    "chalk": [235, 235, 230, 255],
    "chert": [160, 140, 120, 255],
    "cinnabar": [160, 50, 50, 255],
    "diorite": [180, 180, 185, 255],
    "gabbro": [90, 95, 95, 255],
    "granite": [150, 105, 90, 255],
    "karst": [140, 145, 140, 255],
    "limestone": [195, 185, 165, 255],
    "marl": [170, 165, 150, 255],
    "pitchstone": [60, 70, 65, 255],
    "porphyry": [140, 95, 105, 255],
    "red_sandstone": [185, 95, 45, 255],
    "sandstone": [215, 205, 150, 255],
    "scoria": [105, 60, 55, 255],
    "serpentine": [70, 115, 85, 255],
    "slate": [80, 85, 95, 255],
    "stone": [122, 128, 133, 255],
    "tuffite": [105, 108, 100, 255],
    "white_sandstone": [230, 225, 210, 255],
}

rocky_blocks = []
for rname, rslug in rock_types:
    u_name = rname.replace(" ", "_")
    cobble_tex = "cobbled_tuff" if rslug == "tuffite" else f"cobbled_{rslug}"
    base_col = rock_colors.get(rslug, [130, 130, 130, 255])
    moss_col = [(base_col[0] * 3 + 80) // 4, (base_col[1] * 3 + 130) // 4, (base_col[2] * 3 + 60) // 4, 255]
    rocky_blocks.append((f"Rock_{u_name}", f"rock_{rslug}", rname, base_col))
    rocky_blocks.append((f"Cobbled_{u_name}", cobble_tex, f"Cobbled {rname}", base_col))
    rocky_blocks.append((f"Mossy_{u_name}", f"rock_{rslug}", f"Mossy {rname}", moss_col))
    rocky_blocks.append((f"Mossy_Cobbled_{u_name}", cobble_tex, f"Mossy Cobbled {rname}", moss_col))

categories = [
    ("Testing / Debug Blocks", testing_blocks),
    ("Liquid / Fluid Blocks", liquid_blocks),
    ("Frost Blocks", frost_blocks),
    ("Soil Blocks", soil_blocks),
    ("Rocky Blocks", rocky_blocks),
    ("Special Rocky Blocks", special_rocks),
    ("Wood Blocks", wood_blocks),
    ("Aquatic Blocks", aqua_blocks),
    ("Light Source Blocks", light_blocks),
    ("Decoration Blocks", deco_blocks),
]

all_blocks = []
for cat_name, b_list in categories:
    for b in b_list:
        all_blocks.append(b)

assert len(all_blocks) == 238

# Now build src/world/block.rs
def generate_rust():
    # Read the BlockShape implementation from current block.rs
    with open('src/world/block.rs', 'r', encoding='utf-8') as f:
        old_content = f.read()
    
    # Extract up to start of pub enum Voxel
    idx = old_content.find('pub enum Voxel {')
    if idx != -1:
        prefix = old_content[:idx]
        for attr in ['#[allow(non_camel_case_types)]', '#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Reflect)]', '#[repr(u8)]']:
            if attr in prefix:
                prefix = prefix.rsplit(attr, 1)[0]
        block_shape_part = prefix.strip()
    else:
        raise ValueError("Could not find pub enum Voxel")
    
    lines = []
    lines.append(block_shape_part.strip())
    lines.append("")
    lines.append("#[repr(u8)]")
    lines.append("#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Reflect)]")
    lines.append("#[allow(non_camel_case_types)]")
    lines.append("pub enum Voxel {")
    lines.append("    #[default]")
    lines.append("    Air = 0,")
    lines.append("    Occupied = 1,")
    lines.append("    WaterOccupied = 2,")
    lines.append("")
    
    val = 3
    for cat_name, b_list in categories:
        lines.append(f"    // {cat_name} ({len(b_list)})")
        for b in b_list:
            bname = b[0]
            lines.append(f"    {bname} = {val},")
            val += 1
        lines.append("")
    
    lines.append("}")
    lines.append("")
    lines.append("impl Voxel {")
    lines.append("    /// All voxels that map to a texture and are loaded into the terrain texture array.")
    lines.append("    pub const ALL: [Voxel; 238] = [")
    for cat_name, b_list in categories:
        lines.append(f"        // {cat_name}")
        for b in b_list:
            bname = b[0]
            lines.append(f"        Voxel::{bname},")
    lines.append("    ];")
    lines.append("")
    
    # texture_name
    lines.append("    /// The base texture name under `assets/textures/blocks/` without extension.")
    lines.append("    pub fn texture_name(self) -> Option<&'static str> {")
    lines.append("        match self {")
    lines.append("            Self::Air | Self::Occupied | Self::WaterOccupied => None,")
    for cat_name, b_list in categories:
        lines.append(f"            // {cat_name}")
        for b in b_list:
            bname, tex, label, col = b
            lines.append(f'            Self::{bname} => Some("{tex}"),')
    lines.append("        }")
    lines.append("    }")
    lines.append("")

    # side_texture_name
    lines.append("    /// Optional texture override for the side faces (+X, -X, +Z, -Z).")
    lines.append("    /// If None, falls back to `texture_name()`.")
    lines.append("    pub fn side_texture_name(self) -> Option<&'static str> {")
    lines.append("        match self {")
    lines.append('            Self::Rock_Basalt | Self::Mossy_Basalt => Some("rock_basalt_side"),')
    lines.append('            Self::Soil_Grass => Some("soil_grass_side"),')
    lines.append('            Self::Soil_Peat_Grass => Some("soil_peat_grass_side"),')
    lines.append('            Self::Soil_Silt_Grass => Some("soil_silt_grass_side"),')
    lines.append('            Self::Soil_Mulch => Some("soil_mulch_side"),')
    lines.append('            Self::Soil_Peat_Mulch => Some("soil_peat_mulch_side"),')
    lines.append('            Self::Soil_Silt_Mulch => Some("soil_silt_mulch_side"),')
    lines.append('            Self::Soil_Snowy_Grass => Some("soil_snowy_grass_side"),')
    lines.append('            Self::Soil_Snowy_Peat => Some("soil_snowy_peat_side"),')
    lines.append('            Self::Soil_Snowy_Silt => Some("soil_snowy_silt_side"),')
    lines.append('            Self::Tree_Acacia_Log => Some("tree_acacia_bark"),')
    lines.append('            Self::Tree_Birch_Log => Some("tree_birch_bark"),')
    lines.append('            Self::Tree_Charred_Log => Some("tree_charred_bark"),')
    lines.append('            Self::Tree_Dead_Log => Some("tree_dead_bark"),')
    lines.append('            Self::Tree_Mahogany_Log => Some("tree_mahogany_bark"),')
    lines.append('            Self::Tree_Mangrove_Log => Some("tree_mangrove_bark"),')
    lines.append('            Self::Tree_Maple_Log => Some("tree_maple_bark"),')
    lines.append('            Self::Tree_Oak_Log => Some("tree_oak_bark"),')
    lines.append('            Self::Tree_Palm_Log => Some("tree_palm_bark"),')
    lines.append('            Self::Tree_Pine_Log => Some("tree_pine_bark"),')
    lines.append('            Self::Tree_Willow_Log => Some("tree_willow_bark"),')
    lines.append('            Self::Tree_Yew_Log => Some("tree_yew_bark"),')
    lines.append('            Self::Tree_Cactus => Some("tree_cactus_side"),')
    lines.append('            Self::Tree_Mangrove_Roots => Some("tree_mangrove_roots"),')
    lines.append('            Self::Aqua_Algae_Mat => Some("aqua_algae_mat"),')
    lines.append('            Self::Aqua_Geothermal_Vent => Some("aqua_geothermal_vent_side"),')
    lines.append('            Self::Deco_Barrel => Some("deco_barrel"),')
    lines.append('            Self::Deco_Basket => Some("deco_basket_side"),')
    lines.append('            Self::Deco_Bookshelf => Some("deco_bookshelf"),')
    lines.append('            Self::Deco_Bone => Some("deco_bone"),')
    lines.append('            Self::Deco_Hay => Some("deco_hay"),')
    lines.append('            Self::Deco_Thatch => Some("deco_thatch_sides"),')
    lines.append("            _ => None,")
    lines.append("        }")
    lines.append("    }")
    lines.append("")

    # top_texture_name
    lines.append("    /// Optional texture override for the top face (+Y).")
    lines.append("    /// If None, falls back to `texture_name()`.")
    lines.append("    pub fn top_texture_name(self) -> Option<&'static str> {")
    lines.append("        match self {")
    lines.append('            Self::Rock_Basalt | Self::Mossy_Basalt => Some("rock_basalt"),')
    lines.append('            Self::Soil_Grass | Self::Soil_Peat_Grass | Self::Soil_Silt_Grass => Some("soil_grass"),')
    lines.append('            Self::Soil_Mulch => Some("soil_mulch"),')
    lines.append('            Self::Soil_Peat_Mulch => Some("soil_peat_mulch"),')
    lines.append('            Self::Soil_Silt_Mulch => Some("soil_silt_mulch"),')
    lines.append('            Self::Soil_Snowy_Grass | Self::Soil_Snowy_Peat | Self::Soil_Snowy_Silt => Some("soil_snow"),')
    lines.append('            Self::Tree_Acacia_Log => Some("tree_acacia_log"),')
    lines.append('            Self::Tree_Birch_Log => Some("tree_birch_log"),')
    lines.append('            Self::Tree_Charred_Log => Some("tree_charred_log"),')
    lines.append('            Self::Tree_Dead_Log => Some("tree_dead_log"),')
    lines.append('            Self::Tree_Mahogany_Log => Some("tree_mahogany_log"),')
    lines.append('            Self::Tree_Mangrove_Log => Some("tree_mangrove_log"),')
    lines.append('            Self::Tree_Maple_Log => Some("tree_maple_log"),')
    lines.append('            Self::Tree_Oak_Log => Some("tree_oak_log"),')
    lines.append('            Self::Tree_Palm_Log => Some("tree_palm_log"),')
    lines.append('            Self::Tree_Pine_Log => Some("tree_pine_log"),')
    lines.append('            Self::Tree_Willow_Log => Some("tree_willow_log"),')
    lines.append('            Self::Tree_Yew_Log => Some("tree_yew_log"),')
    lines.append('            Self::Tree_Cactus => Some("tree_cactus_top"),')
    lines.append('            Self::Tree_Mangrove_Roots => Some("tree_mangrove_roots_top"),')
    lines.append('            Self::Aqua_Algae_Mat => Some("aqua_algae_mat_top"),')
    lines.append('            Self::Aqua_Geothermal_Vent => Some("aqua_geothermal_vent"),')
    lines.append('            Self::Deco_Barrel => Some("deco_barrel_top"),')
    lines.append('            Self::Deco_Basket => Some("deco_basket_top"),')
    lines.append('            Self::Deco_Bookshelf => Some("deco_bookshelf_top"),')
    lines.append('            Self::Deco_Bone => Some("deco_bone_top"),')
    lines.append('            Self::Deco_Hay => Some("deco_hay_top"),')
    lines.append('            Self::Deco_Thatch => Some("deco_thatch"),')
    lines.append("            _ => None,")
    lines.append("        }")
    lines.append("    }")
    lines.append("")

    # bottom_texture_name
    lines.append("    /// Optional texture override for the bottom face (-Y).")
    lines.append("    /// If None, falls back to `texture_name()`.")
    lines.append("    pub fn bottom_texture_name(self) -> Option<&'static str> {")
    lines.append("        match self {")
    lines.append('            Self::Rock_Basalt | Self::Mossy_Basalt => Some("rock_basalt"),')
    lines.append('            Self::Soil_Grass | Self::Soil_Snowy_Grass | Self::Soil_Mulch => Some("soil_dirt"),')
    lines.append('            Self::Soil_Peat_Grass | Self::Soil_Snowy_Peat | Self::Soil_Peat_Mulch => Some("soil_peat"),')
    lines.append('            Self::Soil_Silt_Grass | Self::Soil_Snowy_Silt | Self::Soil_Silt_Mulch => Some("soil_silt"),')
    lines.append('            Self::Tree_Acacia_Log => Some("tree_acacia_log"),')
    lines.append('            Self::Tree_Birch_Log => Some("tree_birch_log"),')
    lines.append('            Self::Tree_Charred_Log => Some("tree_charred_log"),')
    lines.append('            Self::Tree_Dead_Log => Some("tree_dead_log"),')
    lines.append('            Self::Tree_Mahogany_Log => Some("tree_mahogany_log"),')
    lines.append('            Self::Tree_Mangrove_Log => Some("tree_mangrove_log"),')
    lines.append('            Self::Tree_Maple_Log => Some("tree_maple_log"),')
    lines.append('            Self::Tree_Oak_Log => Some("tree_oak_log"),')
    lines.append('            Self::Tree_Palm_Log => Some("tree_palm_log"),')
    lines.append('            Self::Tree_Pine_Log => Some("tree_pine_log"),')
    lines.append('            Self::Tree_Willow_Log => Some("tree_willow_log"),')
    lines.append('            Self::Tree_Yew_Log => Some("tree_yew_log"),')
    lines.append('            Self::Tree_Cactus => Some("tree_cactus_bot"),')
    lines.append('            Self::Tree_Mangrove_Roots => Some("tree_mangrove_roots_top"),')
    lines.append('            Self::Aqua_Algae_Mat => Some("aqua_algae_mat_top"),')
    lines.append('            Self::Aqua_Geothermal_Vent => Some("aqua_geothermal_vent"),')
    lines.append('            Self::Deco_Barrel => Some("deco_barrel_bot"),')
    lines.append('            Self::Deco_Basket => Some("deco_basket_bot"),')
    lines.append('            Self::Deco_Bookshelf => Some("deco_bookshelf_top"),')
    lines.append('            Self::Deco_Bone => Some("deco_bone_top"),')
    lines.append('            Self::Deco_Hay => Some("deco_hay_top"),')
    lines.append('            Self::Deco_Thatch => Some("deco_thatch"),')
    lines.append("            _ => None,")
    lines.append("        }")
    lines.append("    }")
    lines.append("")

    # overlay_texture_name
    lines.append("    /// Optional overlay texture (like mossy rock or mossy cobbled layers) rendered on top of the base texture.")
    lines.append("    pub fn overlay_texture_name(&self) -> Option<&'static str> {")
    lines.append("        match self {")
    
    # 24 mossy rock variants
    mossy_rocks = [b[0] for b in rocky_blocks if b[0].startswith("Mossy_") and not b[0].startswith("Mossy_Cobbled_")]
    mossy_arms = " |\n            ".join(f"Self::{m}" for m in mossy_rocks)
    lines.append(f"            {mossy_arms} => Some(\"mossy_rock_overlay\"),")
    lines.append("")

    # 24 mossy cobbled variants
    mossy_cobbled = [b[0] for b in rocky_blocks if b[0].startswith("Mossy_Cobbled_")]
    mossy_cobbled_arms = " |\n            ".join(f"Self::{m}" for m in mossy_cobbled)
    lines.append(f"            {mossy_cobbled_arms} => Some(\"mossy_cobbled_overlay\"),")
    lines.append("")
    lines.append("            _ => None,")
    lines.append("        }")
    lines.append("    }")
    lines.append("")

    # is_tinted
    lines.append("    /// Whether this voxel receives custom biome or foliage tinting.")
    lines.append("    pub fn is_tinted(self) -> bool {")
    lines.append("        matches!(")
    lines.append("            self,")
    lines.append("            Self::Soil_Grass")
    lines.append("                | Self::Soil_Peat_Grass")
    lines.append("                | Self::Soil_Silt_Grass")
    lines.append("                | Self::Soil_Snowy_Grass")
    lines.append("                | Self::Liquid_Water")
    lines.append("                | Self::WaterOccupied")
    lines.append("                | Self::Tree_Acacia_Leaves")
    lines.append("                | Self::Tree_Birch_Leaves")
    lines.append("                | Self::Tree_Mahogany_Leaves")
    lines.append("                | Self::Tree_Mangrove_Leaves")
    lines.append("                | Self::Tree_Oak_Leaves")
    lines.append("                | Self::Tree_Palm_Leaves")
    lines.append("                | Self::Tree_Pine_Leaves")
    lines.append("                | Self::Tree_Willow_Leaves")
    lines.append("                | Self::Tree_Yew_Leaves")
    lines.append("        )")
    lines.append("    }")
    lines.append("")

    # tint_color
    lines.append("    /// Color tint applied to vertices for biome / atmospheric coloring.")
    lines.append("    pub fn tint_color(self) -> [f32; 4] {")
    lines.append("        match self {")
    lines.append("            Self::Soil_Grass | Self::Soil_Peat_Grass | Self::Soil_Silt_Grass => [0.55, 0.94, 0.42, 1.0],")
    lines.append("            Self::Soil_Snowy_Grass => [0.90, 0.95, 1.0, 1.0],")
    lines.append("            Self::Liquid_Water | Self::WaterOccupied => [0.35, 0.65, 0.92, 1.0],")
    lines.append("            Self::Tree_Oak_Leaves => [0.60, 1.15, 0.35, 1.0],")
    lines.append("            Self::Tree_Birch_Leaves => [0.85, 1.25, 0.40, 1.0],")
    lines.append("            Self::Tree_Pine_Leaves => [0.40, 0.90, 0.55, 1.0],")
    lines.append("            Self::Tree_Acacia_Leaves => [0.72, 0.92, 0.28, 1.0],")
    lines.append("            Self::Tree_Mahogany_Leaves => [0.45, 1.00, 0.40, 1.0],")
    lines.append("            Self::Tree_Mangrove_Leaves => [0.40, 0.95, 0.42, 1.0],")
    lines.append("            Self::Tree_Palm_Leaves => [0.55, 1.05, 0.30, 1.0],")
    lines.append("            Self::Tree_Willow_Leaves => [0.52, 0.88, 0.48, 1.0],")
    lines.append("            Self::Tree_Yew_Leaves => [0.28, 0.70, 0.38, 1.0],")
    lines.append("            _ => [1.0, 1.0, 1.0, 1.0],")
    lines.append("        }")
    lines.append("    }")
    lines.append("")

    # tint_color_at
    lines.append("    /// Biome-aware blended color tint at a specific world coordinate.")
    lines.append("    pub fn tint_color_at(self, world_voxel: IVec3) -> [f32; 4] {")
    lines.append("        if !self.is_tinted() {")
    lines.append("            return [1.0, 1.0, 1.0, 1.0];")
    lines.append("        }")
    lines.append("        crate::generation::sample_blended_biome_color(")
    lines.append("            self,")
    lines.append("            world_voxel.x as f32,")
    lines.append("            world_voxel.z as f32,")
    lines.append("            1337,")
    lines.append("        )")
    lines.append("    }")
    lines.append("")

    # fallback_color
    lines.append("    /// Fallback 1x1 solid RGBA pixel if the texture file is not found on disk.")
    lines.append("    pub fn fallback_color(self) -> [u8; 4] {")
    lines.append("        match self {")
    lines.append("            Self::Air | Self::Occupied | Self::WaterOccupied => [0, 0, 0, 0],")
    for cat_name, b_list in categories:
        lines.append(f"            // {cat_name}")
        for b in b_list:
            bname, tex, label, col = b
            lines.append(f"            Self::{bname} => [{col[0]}, {col[1]}, {col[2]}, {col[3]}],")
    lines.append("        }")
    lines.append("    }")
    lines.append("")

    # is_empty
    lines.append("    pub fn is_empty(self) -> bool {")
    lines.append("        self == Self::Air")
    lines.append("    }")
    lines.append("")

    # is_water
    lines.append("    pub fn is_water(self) -> bool {")
    lines.append("        matches!(self, Self::Liquid_Water | Self::WaterOccupied)")
    lines.append("    }")
    lines.append("")

    # is_fluid
    lines.append("    pub fn is_fluid(self) -> bool {")
    lines.append("        self.is_water()")
    lines.append("            || matches!(")
    lines.append("                self,")
    lines.append("                Self::Liquid_Acid")
    lines.append("                    | Self::Liquid_Blood")
    lines.append("                    | Self::Liquid_Lava")
    lines.append("                    | Self::Liquid_Molten")
    lines.append("                    | Self::Liquid_Ooze")
    lines.append("                    | Self::Liquid_Sludge")
    lines.append("                    | Self::Liquid_Tar")
    lines.append("                    | Self::Null_Liquid")
    lines.append("            )")
    lines.append("    }")
    lines.append("")

    # is_collidable
    lines.append("    pub fn is_collidable(self) -> bool {")
    lines.append("        !self.is_empty()")
    lines.append("            && !self.is_fluid()")
    lines.append("            && self != Self::Occupied")
    lines.append("            && self != Self::WaterOccupied")
    lines.append("    }")
    lines.append("")

    # is_transparent
    lines.append("    pub fn is_transparent(self) -> bool {")
    lines.append("        self.is_water()")
    lines.append("            || self == Self::Deco_Glass")
    lines.append("            || matches!(self, Self::Frost_Ice | Self::Frost_Fragile_Ice)")
    lines.append("    }")
    lines.append("")

    # is_leaves
    lines.append("    pub fn is_leaves(self) -> bool {")
    lines.append("        matches!(")
    lines.append("            self,")
    lines.append("            Self::Tree_Acacia_Leaves")
    lines.append("                | Self::Tree_Birch_Leaves")
    lines.append("                | Self::Tree_Mahogany_Leaves")
    lines.append("                | Self::Tree_Mangrove_Leaves")
    lines.append("                | Self::Tree_Maple_Leaves_Red")
    lines.append("                | Self::Tree_Maple_Leaves_Orange")
    lines.append("                | Self::Tree_Maple_Leaves_Yellow")
    lines.append("                | Self::Tree_Oak_Leaves")
    lines.append("                | Self::Tree_Palm_Leaves")
    lines.append("                | Self::Tree_Pine_Leaves")
    lines.append("                | Self::Tree_Willow_Leaves")
    lines.append("                | Self::Tree_Yew_Leaves")
    lines.append("        )")
    lines.append("    }")
    lines.append("")

    # is_solid_opaque
    lines.append("    /// Solid opaque blocks that completely occlude light and adjacent faces (not leaves, transparent, fluid, or air).")
    lines.append("    pub fn is_solid_opaque(self) -> bool {")
    lines.append("        !self.is_empty()")
    lines.append("            && !self.is_fluid()")
    lines.append("            && !self.is_leaves()")
    lines.append("            && !self.is_transparent()")
    lines.append("            && self != Self::Occupied")
    lines.append("            && self != Self::WaterOccupied")
    lines.append("    }")
    lines.append("")

    # is_light
    lines.append("    pub fn is_light(self) -> bool {")
    lines.append("        matches!(")
    lines.append("            self,")
    lines.append("            Self::Emit_Blue_Light")
    lines.append("                | Self::Emit_Blue_Torch")
    lines.append("                | Self::Emit_Cold_Light")
    lines.append("                | Self::Emit_Green_Light")
    lines.append("                | Self::Emit_Green_Torch")
    lines.append("                | Self::Emit_Red_Light")
    lines.append("                | Self::Emit_Red_Torch")
    lines.append("                | Self::Emit_Warm_Light")
    lines.append("                | Self::Liquid_Lava")
    lines.append("                | Self::Liquid_Molten")
    lines.append("                | Self::Rock_Magma")
    lines.append("        )")
    lines.append("    }")
    lines.append("")

    # light_color
    lines.append("    pub fn light_color(self) -> Color {")
    lines.append("        match self {")
    lines.append("            Self::Emit_Warm_Light => Color::srgb(1.0, 0.82, 0.42),")
    lines.append("            Self::Emit_Cold_Light => Color::srgb(0.80, 0.90, 1.0),")
    lines.append("            Self::Emit_Red_Light | Self::Emit_Red_Torch => Color::srgb(1.0, 0.25, 0.25),")
    lines.append("            Self::Emit_Green_Light | Self::Emit_Green_Torch => Color::srgb(0.25, 1.0, 0.25),")
    lines.append("            Self::Emit_Blue_Light | Self::Emit_Blue_Torch => Color::srgb(0.25, 0.50, 1.0),")
    lines.append("            Self::Liquid_Lava | Self::Liquid_Molten | Self::Rock_Magma => Color::srgb(1.0, 0.45, 0.15),")
    lines.append("            _ => Color::WHITE,")
    lines.append("        }")
    lines.append("    }")
    lines.append("")

    # label
    lines.append("    pub fn label(self) -> &'static str {")
    lines.append("        match self {")
    lines.append('            Self::Air => "Air",')
    lines.append('            Self::Occupied => "Occupied",')
    lines.append('            Self::WaterOccupied => "Waterlogged Occupied",')
    for cat_name, b_list in categories:
        lines.append(f"            // {cat_name}")
        for b in b_list:
            bname, tex, label, col = b
            lines.append(f'            Self::{bname} => "{label}",')
    lines.append("        }")
    lines.append("    }")
    lines.append("")

    # is_unbreakable
    lines.append("    /// Whether this voxel is completely unbreakable (like bedrock).")
    lines.append("    pub fn is_unbreakable(self) -> bool {")
    lines.append("        self == Self::Rock_Dreadstone")
    lines.append("    }")
    lines.append("}")
    lines.append("")

    return "\n".join(lines)

code = generate_rust()
with open('src/world/block.rs', 'w', encoding='utf-8') as f:
    f.write(code)

print("Successfully generated src/world/block.rs!")
