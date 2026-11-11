#[cfg(test)]
mod tests {
    use common::{
        SkillSetBuilder,
        comp::{
            CharacterActivity, CharacterState, Controller, Energy, Ori, PhysicsState, Poise, Pos,
            Stats, Vel, item::MaterialStatManifest, tool::AbilityMap,
        },
        resources::{DeltaTime, GameMode, Time},
        shared_server_config::ServerConstants,
        terrain::{MapSizeLg, TerrainChunk},
        uid::Uid,
        util::Dir,
    };
    use common_ecs::dispatch;
    use common_state::State;
    use rand::rng;
    use specs::{Builder, Entity, WorldExt};
    use std::{num::NonZeroU64, sync::Arc, time::Duration};
    use vek::{Vec2, Vec3, approx::AbsDiffEq};
    use veloren_common_systems::character_behavior;

    const DEFAULT_WORLD_CHUNKS_LG: MapSizeLg =
        if let Ok(map_size_lg) = MapSizeLg::new(Vec2 { x: 1, y: 1 }) {
            map_size_lg
        } else {
            panic!("Default world chunk size does not satisfy required invariants.");
        };

    fn setup() -> State {
        let pools = State::pools(GameMode::Server);
        let mut state = State::new(
            GameMode::Server,
            pools,
            DEFAULT_WORLD_CHUNKS_LG,
            Arc::new(TerrainChunk::water(0)),
            |dispatch_builder| {
                dispatch::<character_behavior::Sys>(dispatch_builder, &[]);
            },
            #[cfg(feature = "plugins")]
            common_state::plugin::PluginMgr::default(),
        );
        let msm = MaterialStatManifest::load().cloned();
        state.ecs_mut().insert(msm);
        state.ecs_mut().insert(AbilityMap::load().cloned());
        state.ecs_mut().read_resource::<Time>();
        state.ecs_mut().read_resource::<DeltaTime>();
        state
    }

    fn create_entity(state: &mut State, ori: Ori) -> Entity {
        let body = common::comp::Body::Humanoid(common::comp::humanoid::Body::random_with(
            &mut rng(),
            &common::comp::humanoid::Species::Human,
        ));
        let skill_set = SkillSetBuilder::default().build();
        state
            .ecs_mut()
            .create_entity()
            .with(CharacterState::Idle(common::states::idle::Data::default()))
            .with(CharacterActivity::default())
            .with(Pos(Vec3::zero()))
            .with(Vel::default())
            .with(ori)
            .with(body.mass())
            .with(body.density())
            .with(body)
            .with(Energy::new(body))
            .with(Controller::default())
            .with(Poise::new(body))
            .with(skill_set)
            .with(PhysicsState::default())
            .with(Stats::empty(body))
            .with(Uid(NonZeroU64::new(1).unwrap()))
            .build()
    }

    fn tick(state: &mut State, dt: Duration) {
        state.tick(
            dt,
            false,
            None,
            &ServerConstants {
                day_cycle_coefficient: 24.0,
            },
            |_, _| {},
        );
    }

    #[test]
    fn orientation_shortcut() {
        let mut state = setup();
        const TESTCASES: usize = 5;
        let testcases: [(Vec3<f32>, Vec3<f32>); TESTCASES] = [
            // horizontal is unchanged
            (Vec3::unit_x(), Vec3::unit_x()),
            // nearly vertical takes time to adjust
            (
                Vec3::new(0.1, 0.1, 1.0),
                Vec3::new(0.13193887, 0.13193902, 0.98243797),
            ),
            // intermediate case
            (
                Vec3::new(0.6, 0.6, 0.1),
                Vec3::new(0.70256203, 0.70256203, 0.11319534),
            ),
            // edge case: nearly horizontal after system
            (
                Vec3::new(0.6, 0.6, 0.0556),
                Vec3::new(0.7056918, 0.70569193, 0.063229896),
            ),
            // small enough to be horizontal in one step
            (
                Vec3::new(0.6, 0.6, 0.04),
                Vec3::new(0.70637333, 0.70637333, 0.04553493),
            ),
        ];
        let mut entities: [Option<Entity>; TESTCASES] = [None; TESTCASES];
        for i in 0..TESTCASES {
            entities[i] = Some(create_entity(
                &mut state,
                Ori::from_unnormalized_vec(testcases[i].0).unwrap_or_default(),
            ));
        }
        tick(&mut state, Duration::from_secs_f32(0.033));
        let results = state.ecs().read_storage::<Ori>();
        for i in 0..TESTCASES {
            if let Some(e) = entities[i] {
                let result = Dir::from(*results.get(e).expect("Ori missing"));
                assert!(
                    result.abs_diff_eq(&testcases[i].1, 0.005),
                    "{:?} != {:?}",
                    result,
                    testcases[i].1
                );
            }
        }
    }

    // Icebreaker : jalon A — les touches 1 à 3 d'un personnage du registre lancent nos compétences.
    mod icebreaker {
        use super::*;
        use common::{
            comp::{
                Combo, ControlAction, InputKind, Inventory, Item, ability::Ability,
                inventory::loadout_builder::LoadoutBuilder,
            },
            event::{ChangeStanceEvent, EventBus},
            icebreaker_competences::{Acquis, IcebreakerCompetences, TOUCHE_ULTIME, registre},
        };

        /// Un guerrier du registre, arme sortie, avec les armes du jeu d'armes `jeu` et le lot par
        /// défaut de sa classe (grande épée : balayage lourd, tourbillon ; ultime : fureur).
        fn guerrier(state: &mut State, jeu: &str) -> Entity {
            let armes = &registre().jeu(jeu).expect("jeu d'armes").armes;
            let loadout = LoadoutBuilder::empty()
                .active_mainhand(Some(Item::new_from_asset_expect(&armes.0)))
                .active_offhand(armes.1.as_deref().map(Item::new_from_asset_expect))
                .build();
            let entity = create_entity(state, Ori::default());
            let ecs = state.ecs_mut();
            ecs.write_storage()
                .insert(entity, Inventory::with_loadout_humanoid(loadout))
                .unwrap();
            // Comme tout joueur : le compteur de combo, que certaines capacités consultent.
            ecs.write_storage().insert(entity, Combo::default()).unwrap();
            ecs.write_storage()
                .insert(
                    entity,
                    IcebreakerCompetences::nouveau("guerrier", Acquis::par_defaut("guerrier"))
                        .expect("le guerrier est décrit par nos données"),
                )
                .unwrap();
            ecs.write_storage()
                .insert(
                    entity,
                    CharacterState::Wielding(common::states::wielding::Data { is_sneaking: false }),
                )
                .unwrap();
            entity
        }

        fn appuyer(state: &mut State, entity: Entity, touche: usize) {
            state
                .ecs()
                .write_storage::<Controller>()
                .get_mut(entity)
                .expect("Controller")
                .push_action(ControlAction::basic_input(InputKind::Ability(touche)));
        }

        /// La compétence en cours, si le personnage en lance une.
        fn lancee(state: &State, entity: Entity) -> Option<Ability> {
            state
                .ecs()
                .read_storage::<CharacterState>()
                .get(entity)
                .and_then(|character| character.ability_info())
                .and_then(|info| info.ability)
                .map(|specified| specified.ability)
        }

        fn numero(id: &str) -> u16 { registre().numero(id).unwrap_or_else(|| panic!("{id}")) }

        #[test]
        fn icebreaker_la_touche_1_lance_la_competence_placee() {
            let mut state = setup();
            let entity = guerrier(&mut state, "grande_epee");
            // Le banc n'a pas ce bus : sans lui, l'entrée en posture serait perdue sans bruit.
            state
                .ecs_mut()
                .insert(EventBus::<ChangeStanceEvent>::default());
            appuyer(&mut state, entity, 0);
            tick(&mut state, Duration::from_secs_f32(0.033));

            // Le balayage lourd part : un `ComboMelee2`, qui porte notre numéro (le client en tire
            // l'animation de la capacité de Veloren).
            assert_eq!(
                lancee(&state, entity),
                Some(Ability::Icebreaker(numero("balayage_lourd")))
            );
            let characters = state.ecs().read_storage::<CharacterState>();
            let character = characters.get(entity).expect("CharacterState");
            assert!(matches!(character, CharacterState::ComboMelee2(_)), "{character:?}");
            let inventories = state.ecs().read_storage::<Inventory>();
            assert_eq!(
                character
                    .ability_info()
                    .and_then(|info| info.ability)
                    .and_then(|ability| ability.ability_id(Some(character), inventories.get(entity))),
                Some("common.abilities.sword.heavy_sweep")
            );
            // Pas de posture : la capacité de Veloren en demande une, elle est ignorée.
            assert_eq!(
                state
                    .ecs()
                    .read_resource::<EventBus<ChangeStanceEvent>>()
                    .recv_all()
                    .len(),
                0
            );
        }

        #[test]
        fn icebreaker_une_arme_non_apprise_ne_garde_que_l_ultime() {
            let mut state = setup();
            // Le lot par défaut ne connaît que la grande épée.
            let entity = guerrier(&mut state, "marteau");
            appuyer(&mut state, entity, 0);
            tick(&mut state, Duration::from_secs_f32(0.033));
            assert_eq!(lancee(&state, entity), None, "touche 1 vide");
            assert!(matches!(
                state.ecs().read_storage::<CharacterState>().get(entity),
                Some(CharacterState::Wielding(_))
            ));

            // L'ultime est à la classe : il part quelle que soit l'arme.
            state
                .ecs()
                .write_storage::<Controller>()
                .get_mut(entity)
                .expect("Controller")
                .queued_inputs
                .clear();
            appuyer(&mut state, entity, TOUCHE_ULTIME);
            tick(&mut state, Duration::from_secs_f32(0.033));
            assert_eq!(lancee(&state, entity), Some(Ability::Icebreaker(numero("fureur"))));
        }
    }
}
