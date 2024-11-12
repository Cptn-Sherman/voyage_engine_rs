

// todo: finish this section.

// fn update_flycam (
//     time: Res<Time>,
//     mut query: Query<(&mut Transform, &FlyCam)>,
// ) {
//     for (mut transform, _flycam) in query.iter_mut() {
//         let mut translation = Vec3::ZERO;
//         let mut rotation = Quat::IDENTITY;

//         // Translation
//         if Input::is_key_down(KeyCode::W) {
//             translation += Vec3::Y;
//         }
//         if Input::is_key_down(KeyCode::S) {
//             translation -= Vec3::Y;
//         }
//         if Input::is_key_down(KeyCode::A) {
//             translation -= Vec3::X;
//         }
//         if Input::is_key_down(KeyCode::D) {
//             translation += Vec3::X;
//         }
//         if Input::is_key_down(KeyCode::Q) {
//             translation -= Vec3::Z;
//         }
//         if Input::is_key_down(KeyCode::E) {
//             translation += Vec3::Z;
//         }

//         // Rotation
//         if Input::is_mouse_button_down(MouseButton::Right) {
//             let delta = Input::mouse_delta();
//             rotation = Quat::from_rotation_y(-delta.x.to_radians() * 0.5)
//                 * Quat::from_rotation_x(-delta.y.to_radians() * 0.5);
//         }

//         transform.translation += rotation.mul_vec3(translation) * 5.0 * time.delta_seconds();
//         transform.rotation = rotation * transform.rotation;
//     }
// }