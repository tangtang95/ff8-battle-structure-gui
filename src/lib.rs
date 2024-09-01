use std::io::Cursor;

use binrw::prelude::*;

#[derive(BinRead, Debug)]
pub struct PackedBattleStructure {
    stage_id: u8,
    flags: u8,
    main_camera: u8,
    secondary_camera: u8,
    visible_enemies: u8,
    loaded_enemies: u8,
    targetable_enemies: u8,
    enabled_enemies: u8,
    enemies_coords: [Coordinate; 8],
    id_enemies: [u8; 8],
    unknown_1: [u16; 8],
    unknown_2: [u16; 8],
    unknown_3: [u16; 8],
    unknown_4: [u8; 8],
    enemy_level: [u8; 8],
}

#[derive(BinRead, Debug, Clone)]
pub struct Coordinate {
    x: i16,
    y: i16,
    z: i16,
}

#[derive(Debug)]
pub struct BattleStructure {
    stage_id: u8,
    flags: BattleFlags,
    main_camera: CameraAttributes,
    secondary_camera: CameraAttributes,
    enemies: [Enemy; 8],
}

/// Flags ordered from LSB to MSB
#[derive(Debug)]
pub struct BattleFlags {
    cannot_escape: bool,
    disable_win_fanfare: bool,
    show_timer: bool,
    no_exp: bool,
    disable_exp_screen: bool,
    force_surprise_attack: bool,
    force_back_attack: bool,
    scripted_battle: bool,
}

#[derive(Debug)]
pub struct CameraAttributes {
    /// camera number of size u4
    number: u8,
    /// camera animation of size u4
    animation: u8,
}

/// Enemy information where id is equal to PackedBattleStructure.id_enemies[idx] - 0x10
#[derive(Debug)]
pub struct Enemy {
    id: u8,
    level: u8,
    enabled: bool,
    visible: bool,
    loaded: bool,
    targetable: bool,
    coordinate: Coordinate,
    unknown_1: u16,
    unknown_2: u16,
    unknown_3: u16,
    unknown_4: u8,
}

pub fn parse_battle_structure(bytes: &[u8]) -> anyhow::Result<PackedBattleStructure> {
    let mut cursor = Cursor::new(bytes);
    let battle_structure_packed: PackedBattleStructure = cursor.read_le()?;
    Ok(battle_structure_packed)
}

impl From<u8> for BattleFlags {
    fn from(value: u8) -> Self {
        Self {
            cannot_escape: (value & (1 << 0)) > 0,
            disable_win_fanfare: (value & (1 << 1)) > 0,
            show_timer: (value & (1 << 2)) > 0,
            no_exp: (value & (1 << 3)) > 0,
            disable_exp_screen: (value & (1 << 4)) > 0,
            force_surprise_attack: (value & (1 << 5)) > 0,
            force_back_attack: (value & (1 << 6)) > 0,
            scripted_battle: (value & (1 << 7)) > 0,
        }
    }
}

impl From<u8> for CameraAttributes {
    fn from(value: u8) -> Self {
        Self {
            number: value >> 4,
            animation: value & 0xF,
        }
    }
}

impl From<PackedBattleStructure> for BattleStructure {
    fn from(value: PackedBattleStructure) -> Self {

        BattleStructure {
            stage_id: value.stage_id,
            flags: value.flags.into(),
            main_camera: value.main_camera.into(),
            secondary_camera: value.secondary_camera.into(),
            enemies: [
                value.enemy(0),
                value.enemy(1),
                value.enemy(2),
                value.enemy(3),
                value.enemy(4),
                value.enemy(5),
                value.enemy(6),
                value.enemy(7),
            ]
        }
    }
}

impl PackedBattleStructure {
    pub fn enemy(&self, index: usize) -> Enemy {
        let mask = 0x80 >> index;

        Enemy {
            id: self.id_enemies[index] - 0x10,
            level: self.enemy_level[index],
            enabled: (self.enabled_enemies & mask) > 0,
            loaded: (self.loaded_enemies & mask) > 0,
            visible: (self.visible_enemies & mask) > 0,
            targetable: (self.targetable_enemies & mask) > 0,
            coordinate: self.enemies_coords[index].clone(),
            unknown_1: self.unknown_1[index],
            unknown_2: self.unknown_2[index],
            unknown_3: self.unknown_3[index],
            unknown_4: self.unknown_4[index],
        }
    }
}

#[cfg(test)]
mod test {
    use hex_literal::hex;

    use super::*;

    const BYTES: &[u8] = &hex!(
        "0681 0013 0000 0080 4c04 0000 1cf3 9cff"
        "0000 bce9 9001 0000 bce9 74f5 0000 bce9"
        "5cf9 0000 bce9 a8fd 0000 bce9 68f7 0000"
        "bce9 50fb 0000 bce9 5710 1010 1010 1010"
        "707f c800 c800 c800 c800 c800 c800 c800"
        "1701 c800 c800 c800 c800 c800 c800 c800"
        "9004 60ea 60ea 60ea 60ea 60ea 60ea 60ea"
        "0102 0202 0202 0202 ffff ffff ffff ffff"
    );

    #[test]
    fn verify_pattle_battle_structure_layout() {
        assert_eq!(size_of::<PackedBattleStructure>(), 128);
    }

    #[test]
    fn verify_coordinate_layout() {
        assert_eq!(size_of::<Coordinate>(), 6);
    }

    #[test]
    fn test_parser() {
        let packed_battle_structure = parse_battle_structure(BYTES).unwrap();
        println!("{:?}", packed_battle_structure);
        assert_eq!(packed_battle_structure.stage_id, 6);
    }

    #[test]
    fn test_parse_battle_structure() {
        let packed_battle_structure = parse_battle_structure(BYTES).unwrap();
        let battle_structure = BattleStructure::from(packed_battle_structure);
        println!("{:?}", battle_structure);
        assert_eq!(battle_structure.stage_id, 6);
    }
}




