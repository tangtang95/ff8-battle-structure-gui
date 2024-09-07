use std::io::Cursor;

use binrw::{BinRead, BinWrite};

#[derive(BinRead, BinWrite, Debug)]
#[brw(little)]
pub struct PackedBattleStructure {
    stage_id: u8,
    flags: u8,
    main_camera: u8,
    secondary_camera: u8,
    not_visible_enemies: u8,
    not_loaded_enemies: u8,
    not_targetable_enemies: u8,
    enabled_enemies: u8,
    enemies_coords: [Coordinate; 8],
    id_enemies: [u8; 8],
    unknown_1: [u16; 8],
    unknown_2: [u16; 8],
    unknown_3: [u16; 8],
    unknown_4: [u8; 8],
    enemy_level: [u8; 8],
}

#[derive(BinRead, BinWrite, Debug, Clone)]
#[brw(little)]
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
#[derive(Debug, Clone)]
pub struct Enemy {
    id: u8,
    level: u8,
    enabled: bool,
    invisible: bool,
    not_loaded: bool,
    untargetable: bool,
    coordinate: Coordinate,
    unknown_1: u16,
    unknown_2: u16,
    unknown_3: u16,
    unknown_4: u8,
}

impl PackedBattleStructure {

    pub fn try_from_bytes(bytes: &[u8]) -> anyhow::Result<PackedBattleStructure> {
        let mut cursor = Cursor::new(bytes);
        let battle_structure_packed = PackedBattleStructure::read(&mut cursor)?;
        Ok(battle_structure_packed)
    }

    pub fn into_battle_structure(self) -> BattleStructure {
        BattleStructure {
            stage_id: self.stage_id,
            flags: self.battle_flags(),
            main_camera: self.main_camera(),
            secondary_camera: self.secondary_camera(),
            enemies: [
                self.enemy(0),
                self.enemy(1),
                self.enemy(2),
                self.enemy(3),
                self.enemy(4),
                self.enemy(5),
                self.enemy(6),
                self.enemy(7),
            ]
        }
    }

    fn main_camera(&self) -> CameraAttributes {
        CameraAttributes {
            number: self.main_camera >> 4,
            animation: self.main_camera & 0xF,
        }
    }

    fn secondary_camera(&self) -> CameraAttributes {
        CameraAttributes {
            number: self.secondary_camera >> 4,
            animation: self.secondary_camera & 0xF,
        }
    }

    fn battle_flags(&self) -> BattleFlags {
        BattleFlags {
            cannot_escape: (self.flags & (1 << 0)) > 0,
            disable_win_fanfare: (self.flags & (1 << 1)) > 0,
            show_timer: (self.flags & (1 << 2)) > 0,
            no_exp: (self.flags & (1 << 3)) > 0,
            disable_exp_screen: (self.flags & (1 << 4)) > 0,
            force_surprise_attack: (self.flags & (1 << 5)) > 0,
            force_back_attack: (self.flags & (1 << 6)) > 0,
            scripted_battle: (self.flags & (1 << 7)) > 0,
        }
    }

    fn enemy(&self, index: usize) -> Enemy {
        let mask = 0x80 >> index;

        Enemy {
            id: self.id_enemies[index] - 0x10,
            level: self.enemy_level[index],
            enabled: (self.enabled_enemies & mask) > 0,
            not_loaded: (self.not_loaded_enemies & mask) > 0,
            invisible: (self.not_visible_enemies & mask) > 0,
            untargetable: (self.not_targetable_enemies & mask) > 0,
            coordinate: self.enemies_coords[index].clone(),
            unknown_1: self.unknown_1[index],
            unknown_2: self.unknown_2[index],
            unknown_3: self.unknown_3[index],
            unknown_4: self.unknown_4[index],
        }
    }
}

impl BattleStructure {
    pub fn as_packed_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let packed_battle_structure = PackedBattleStructure {
            stage_id: self.stage_id,
            flags: self.packed_battle_flags(),
            main_camera: (self.main_camera.number << 4) | self.main_camera.animation,
            secondary_camera: (self.secondary_camera.number << 4) | self.secondary_camera.animation,
            not_visible_enemies: self.packed_not_visible_enemies(),
            not_loaded_enemies: self.packed_not_loaded_enemies(),
            not_targetable_enemies: self.packed_untargetable_enemies(),
            enabled_enemies: self.packed_enabled_enemies(),
            enemies_coords: self.enemies.each_ref().map(|enemy| enemy.coordinate.clone()),
            id_enemies: self.enemies.each_ref().map(|enemy| enemy.id + 0x10),
            unknown_1: self.enemies.each_ref().map(|enemy| enemy.unknown_1),
            unknown_2: self.enemies.each_ref().map(|enemy| enemy.unknown_2),
            unknown_3: self.enemies.each_ref().map(|enemy| enemy.unknown_3),
            unknown_4: self.enemies.each_ref().map(|enemy| enemy.unknown_4),
            enemy_level: self.enemies.each_ref().map(|enemy| enemy.level),
        };

        let mut writer = Cursor::new(Vec::new());
        packed_battle_structure.write(&mut writer)?;
        Ok(writer.into_inner())
    }

    fn packed_battle_flags(&self) -> u8 {
        let mut flags = 0u8;
        flags |= self.flags.cannot_escape as u8;
        flags |= (self.flags.disable_win_fanfare as u8) << 1;
        flags |= (self.flags.show_timer as u8) << 2;
        flags |= (self.flags.no_exp as u8) << 3;
        flags |= (self.flags.disable_exp_screen as u8) << 4;
        flags |= (self.flags.force_surprise_attack as u8) << 5;
        flags |= (self.flags.force_back_attack as u8) << 6;
        flags |= (self.flags.scripted_battle as u8) << 7;
        flags
    }

    fn packed_not_visible_enemies(&self) -> u8 {
        let mut not_visible_enemies = 0u8;
        for (i, enemy) in self.enemies.iter().enumerate() {
            not_visible_enemies |= (enemy.invisible as u8) << (self.enemies.len() - 1 - i);
        }
        not_visible_enemies
    }

    fn packed_not_loaded_enemies(&self) -> u8 {
        let mut not_loaded_enemies = 0u8;
        for (i, enemy) in self.enemies.iter().enumerate() {
            not_loaded_enemies |= (enemy.not_loaded as u8) << (self.enemies.len() - 1 - i);
        }
        not_loaded_enemies
    }

    fn packed_enabled_enemies(&self) -> u8 {
        let mut enabled_enemies = 0u8;
        for (i, enemy) in self.enemies.iter().enumerate() {
            enabled_enemies |= (enemy.enabled as u8) << (self.enemies.len() - 1 - i);
        }
        enabled_enemies
    }

    fn packed_untargetable_enemies(&self) -> u8 {
        let mut untargetable_enemies = 0u8;
        for (i, enemy) in self.enemies.iter().enumerate() {
            untargetable_enemies |= (enemy.untargetable as u8) << (self.enemies.len() - 1 - i);
        }
        untargetable_enemies
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
        let packed_battle_structure = PackedBattleStructure::try_from_bytes(BYTES).unwrap();
        println!("{:?}", packed_battle_structure);
        assert_eq!(packed_battle_structure.stage_id, 6);
    }

    #[test]
    fn test_parse_battle_structure() {
        let packed_battle_structure = PackedBattleStructure::try_from_bytes(BYTES).unwrap();
        let battle_structure = packed_battle_structure.into_battle_structure();
        println!("{:?}", battle_structure);
        assert_eq!(battle_structure.stage_id, 6);

        assert!(battle_structure.flags.cannot_escape);
        assert!(battle_structure.flags.scripted_battle);
        assert!(!battle_structure.flags.no_exp);
        assert!(!battle_structure.flags.force_back_attack);
        assert!(!battle_structure.flags.force_surprise_attack);
        assert!(!battle_structure.flags.show_timer);
        assert!(!battle_structure.flags.disable_exp_screen);
        assert!(!battle_structure.flags.disable_win_fanfare);

        assert_eq!(battle_structure.main_camera.number, 0);
        assert_eq!(battle_structure.main_camera.animation, 0);
        assert_eq!(battle_structure.secondary_camera.number, 1);
        assert_eq!(battle_structure.secondary_camera.animation, 3);

        assert_eq!(battle_structure.enemies[0].id, 71);
        assert_eq!(battle_structure.enemies[0].level, 255);
        assert!(battle_structure.enemies[0].enabled);
        assert!(!battle_structure.enemies[0].invisible);
        assert!(!battle_structure.enemies[0].untargetable);
        assert!(!battle_structure.enemies[0].not_loaded);
        assert_eq!(battle_structure.enemies[0].coordinate.x, 1100);
        assert_eq!(battle_structure.enemies[0].coordinate.y, 0);
        assert_eq!(battle_structure.enemies[0].coordinate.z, -3300);
        assert_eq!(battle_structure.enemies[0].unknown_1, 0x7f70);
        assert_eq!(battle_structure.enemies[0].unknown_2, 0x117);
        assert_eq!(battle_structure.enemies[0].unknown_3, 0x490);
        assert_eq!(battle_structure.enemies[0].unknown_4, 0x1);

        assert_eq!(battle_structure.enemies[4].id, 0);
        assert_eq!(battle_structure.enemies[4].level, 255);
        assert!(!battle_structure.enemies[4].enabled);
        assert!(!battle_structure.enemies[4].invisible);
        assert!(!battle_structure.enemies[4].untargetable);
        assert!(!battle_structure.enemies[4].not_loaded);
        assert_eq!(battle_structure.enemies[4].coordinate.x, -1700);
        assert_eq!(battle_structure.enemies[4].coordinate.y, 0);
        assert_eq!(battle_structure.enemies[4].coordinate.z, -5700);
        assert_eq!(battle_structure.enemies[4].unknown_1, 0xc8);
        assert_eq!(battle_structure.enemies[4].unknown_2, 0xc8);
        assert_eq!(battle_structure.enemies[4].unknown_3, 0xea60);
        assert_eq!(battle_structure.enemies[4].unknown_4, 0x2);
    }


    #[test]
    fn test_parser_and_writer() {
        let packed_battle_structure = PackedBattleStructure::try_from_bytes(BYTES).unwrap();
        let battle_structure = packed_battle_structure.into_battle_structure();
        assert_eq!(battle_structure.as_packed_bytes().unwrap(), BYTES);
    }
}




