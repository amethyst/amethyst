//! Module containing the system managing the text editing cursor create, deletion and position.

// TODO: Complete this and remove the logic from the UI Pass. Scheduled for completion after Transform2D.
// File currently ignored because it is not added to lib.rs

/// Tag component placed on the cursor of a text field being edited.
pub struct TextEditingCursor;

/// Manages the text editing cursor create, deletion and position.
pub struct TextEditingCursorSystem;

impl<'a> System for TextEditingCursorSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, UiTransform>,
        ReadStorage<'a, TextEditing>,
        ReadStorage<'a, Parent>,
        ReadStorage<'a, Selected>,
        WriteStorage<'a, Cursor>,
        WriteStorage<'a, Blink>,
        WriteStorage<'a, Handle<Texture>>,
        ReadStorage<'a, UiConfig>,
    );

    fn run(
        &mut self,
        (
            entities,
            mut transforms,
            editings,
            parents,
            selecteds,
            mut cursors,
            mut blinks,
            mut textures,
            colors,
            config,
        ): Self::SystemData,
    ) {
        #[cfg(feature = "profiler")]
        profile_scope!("text_editing_cursor_system");

        // Go through all text editing entities.
        for (entity, _) in (&*entities, &editings).join() {
            // Finds child cursor of current text editing entity.
            let cursor = (&*entities, &parents, &cursors)
                .join()
                .filter(|t| t.1.parent == entity)
                .map(|t| t.0)
                .next();
            let selected = selecteds.contains(entity);

            if let Some(cursor_entity) = cursor {
                if !selected {
                    // Shouldn't have a cursor.
                    entities.delete(cursor_entity);
                    continue;
                }
            } else if selected {
                // TODO: Should have a cursor.
                let cursor_entity = entities.push();
                cursors
                    .insert(cursor_entity, Cursor)
                    .expect("Unreachable: Entity just created.");
                parents
                    .insert(
                        cursor_entity,
                        Parent {
                            parent: entity.clone(),
                        },
                    )
                    .expect("Unreachable: Entity just created.");
                transforms
                    .insert(cursor_entity, UiTransform::new())
                    .expect("Unreachable: Entity just created.");
                blinks
                    .insert(cursor_entity, Blink::new(config.blink_delay))
                    .expect("Unreachable: Entity just created.");
                textures
                    .insert(cursor_entity, config.cursor)
                    .expect("Unreachable: Entity just created.");
            }
            // TODO: Move the cursor to the correct location.
            // TODO: Ajust cursor thicc-ness depending on is block cursor and text char width.
        }
    }
}
