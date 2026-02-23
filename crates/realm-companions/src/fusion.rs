use rand::Rng;
use chrono::Utc;

use crate::companion::{Companion, Rarity};
use crate::names;

#[derive(Debug)]
pub enum FusionError {
    RarityMismatch,
    AlreadyMaxRarity,
    SameCompanion,
    CompanionIsFamiliar,
}

impl std::fmt::Display for FusionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RarityMismatch => write!(f, "both companions must be the same rarity"),
            Self::AlreadyMaxRarity => write!(f, "SS companions cannot be fused further"),
            Self::SameCompanion => write!(f, "cannot fuse a companion with itself"),
            Self::CompanionIsFamiliar => write!(f, "cannot fuse the active familiar"),
        }
    }
}

impl std::error::Error for FusionError {}

pub fn validate_fusion(a: &Companion, b: &Companion) -> Result<Rarity, FusionError> {
    if a.id == b.id {
        return Err(FusionError::SameCompanion);
    }
    if a.rarity != b.rarity {
        return Err(FusionError::RarityMismatch);
    }
    if a.rarity == Rarity::SS {
        return Err(FusionError::AlreadyMaxRarity);
    }
    if a.is_familiar || b.is_familiar {
        return Err(FusionError::CompanionIsFamiliar);
    }
    Ok(a.rarity.next().unwrap())
}

pub fn fuse(a: &Companion, b: &Companion) -> Result<Companion, FusionError> {
    let target_rarity = validate_fusion(a, b)?;
    let mut rng = rand::rng();

    let total_bond = a.bond_xp + b.bond_xp;
    let bond_inheritance_ratio = 0.25;
    let inherited_xp = (total_bond as f64 * bond_inheritance_ratio) as u64;

    let primary = if a.bond_xp >= b.bond_xp { a } else { b };
    let secondary = if a.bond_xp >= b.bond_xp { b } else { a };

    let archetype = if rng.random_bool(0.7) {
        primary.archetype
    } else {
        secondary.archetype
    };

    let dere_type = if rng.random_bool(0.6) {
        primary.dere_type
    } else {
        secondary.dere_type
    };

    let region = if rng.random_bool(0.5) {
        primary.region
    } else {
        secondary.region
    };

    let aesthetic = if rng.random_bool(0.5) {
        primary.aesthetic
    } else {
        secondary.aesthetic
    };

    let name = names::generate(&mut rng, &region);
    let personality_seed: u64 = primary.personality_seed.wrapping_mul(31).wrapping_add(secondary.personality_seed);

    let mut result = Companion {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        archetype,
        dere_type,
        region,
        aesthetic,
        rarity: target_rarity,
        bond_level: 0,
        bond_xp: inherited_xp,
        is_familiar: false,
        familiar_eligible: false,
        created_at: Utc::now(),
        fused_from: Some([a.id.clone(), b.id.clone()]),
        personality_seed,
    };

    let mut level_check = true;
    while level_check {
        level_check = result.add_bond_xp(0);
    }
    loop {
        let next = Companion::bond_xp_for_level(result.bond_level + 1);
        if result.bond_xp >= next {
            result.bond_level += 1;
        } else {
            break;
        }
    }

    if result.rarity >= Rarity::SS && result.bond_level >= 5 {
        result.familiar_eligible = true;
    }

    Ok(result)
}

pub fn fusion_preview_text(a: &Companion, b: &Companion) -> Result<String, FusionError> {
    let target = validate_fusion(a, b)?;
    Ok(format!(
        "Fusion Ritual\n\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\n\
         {a_emoji} {a_name} ({a_rarity})\n\
         {b_emoji} {b_name} ({b_rarity})\n\
         \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\n\
         \u{2192} Result: {target_emoji} {target} rarity\n\
         Bond inheritance: {bond}% of combined XP\n\n\
         \u{26A0} Both companions will be consumed.",
        a_emoji = a.rarity.color_emoji(),
        a_name = a.display_name(),
        a_rarity = a.rarity,
        b_emoji = b.rarity.color_emoji(),
        b_name = b.display_name(),
        b_rarity = b.rarity,
        target_emoji = target.color_emoji(),
        target = target,
        bond = 25,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gacha::{GachaEngine, PityState};

    fn make_companion(rarity: Rarity) -> Companion {
        let engine = GachaEngine::default();
        let mut pity = PityState::default();
        let mut c = engine.pull(&mut pity);
        c.rarity = rarity;
        c
    }

    #[test]
    fn test_basic_fusion() {
        let a = make_companion(Rarity::C);
        let b = make_companion(Rarity::C);
        let result = fuse(&a, &b).unwrap();
        assert_eq!(result.rarity, Rarity::B);
        assert!(result.fused_from.is_some());
    }

    #[test]
    fn test_fusion_rarity_mismatch() {
        let a = make_companion(Rarity::C);
        let b = make_companion(Rarity::B);
        assert!(fuse(&a, &b).is_err());
    }

    #[test]
    fn test_fusion_ss_blocked() {
        let a = make_companion(Rarity::SS);
        let b = make_companion(Rarity::SS);
        assert!(fuse(&a, &b).is_err());
    }

    #[test]
    fn test_fusion_same_companion() {
        let a = make_companion(Rarity::C);
        let b = a.clone();
        assert!(fuse(&a, &b).is_err());
    }

    #[test]
    fn test_bond_inheritance() {
        let mut a = make_companion(Rarity::B);
        let mut b = make_companion(Rarity::B);
        a.bond_xp = 1000;
        b.bond_xp = 500;
        let result = fuse(&a, &b).unwrap();
        assert!(result.bond_xp > 0);
        assert!(result.bond_xp <= 1500);
    }

    #[test]
    fn test_full_chain_c_to_ss() {
        let mut companions: Vec<Companion> = (0..16).map(|_| make_companion(Rarity::C)).collect();

        let mut b_tier: Vec<Companion> = Vec::new();
        while companions.len() >= 2 {
            let b = companions.pop().unwrap();
            let a = companions.pop().unwrap();
            b_tier.push(fuse(&a, &b).unwrap());
        }
        assert_eq!(b_tier.len(), 8);
        assert!(b_tier.iter().all(|c| c.rarity == Rarity::B));

        let mut a_tier: Vec<Companion> = Vec::new();
        while b_tier.len() >= 2 {
            let b = b_tier.pop().unwrap();
            let a = b_tier.pop().unwrap();
            a_tier.push(fuse(&a, &b).unwrap());
        }
        assert_eq!(a_tier.len(), 4);
        assert!(a_tier.iter().all(|c| c.rarity == Rarity::A));

        let mut s_tier: Vec<Companion> = Vec::new();
        while a_tier.len() >= 2 {
            let b = a_tier.pop().unwrap();
            let a = a_tier.pop().unwrap();
            s_tier.push(fuse(&a, &b).unwrap());
        }
        assert_eq!(s_tier.len(), 2);
        assert!(s_tier.iter().all(|c| c.rarity == Rarity::S));

        let ss = fuse(&s_tier[0], &s_tier[1]).unwrap();
        assert_eq!(ss.rarity, Rarity::SS);
    }
}
