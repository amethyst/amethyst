use amethyst_core::cgmath::{Quaternion, Deg, Euler, Vector3};
use amethyst_core::timing::Time;
use amethyst_core::transform::Transform;
use amethyst_input::InputHandler;
use amethyst_renderer::{Camera,ScreenDimensions};
use specs::{Fetch, Join, ReadStorage, System, WriteStorage};

/// The system that manages the camera movement
pub struct FlyCameraMovementSystem<'a>{
    /// The movement speed of the camera in units per second.
    speed: f32,
    /// The name of the input axis to locally move in the x coordinates.
    move_x_name: Option<&'a str>,
    /// The name of the input axis to locally move in the y coordinates.
    move_y_name: Option<&'a str>,
    /// The name of the input axis to locally move in the z coordinates.
    move_z_name: Option<&'a str>,
}

impl<'a> FlyCameraMovementSystem<'a>{
    pub fn new(speed: f32,move_x_name: Option<&'a str>,move_y_name: Option<&'a str>,move_z_name: Option<&'a str>)->Self{
        FlyCameraMovementSystem{
            speed,
            move_x_name,
            move_y_name,
            move_z_name,
        }
    }

    fn get_axis(name: Option<&'a str>,input: &Fetch<InputHandler<String,String>>)->f32{
        if let Some(n) = name{
            if let Some(v) = input.axis_value(n){
                return v as f32;
            }
        }
        return 0.0;
    }
}

impl<'a,'b> System<'a> for FlyCameraMovementSystem<'b> {
    type SystemData = (
        Fetch<'a, Time>,
        ReadStorage<'a, Camera>,
        WriteStorage<'a, Transform>,
        Fetch<'a, InputHandler<String, String>>,
    );

    fn run(&mut self, (time,camera,mut transform,input): Self::SystemData){
        let x = FlyCameraMovementSystem::get_axis(self.move_x_name,&input);
        let y = FlyCameraMovementSystem::get_axis(self.move_y_name,&input);
        let z = FlyCameraMovementSystem::get_axis(self.move_z_name,&input);

        let dir = Vector3::new(x,y,z);

        for (_,transform) in (&camera,&mut transform).join(){
            transform.move_local(dir, time.delta_seconds() * self.speed);
        }
    }
}

/// The system that manages the camera rotation.
pub struct FlyCameraRotationSystem{
    sensitivity_x: f32,
    sensitivity_y: f32,
}

impl FlyCameraRotationSystem{
    pub fn new(sensitivity_x: f32, sensitivity_y: f32)->Self{
        FlyCameraRotationSystem{
            sensitivity_x,
            sensitivity_y,
        }
    }
}

impl<'a> System<'a> for FlyCameraRotationSystem {
    type SystemData = (
        Fetch<'a, InputHandler<String, String>>,
        Fetch<'a, ScreenDimensions>,
        ReadStorage<'a, Camera>,
        WriteStorage<'a, Transform>,
    );

    fn run(&mut self, (input,dim,camera,mut transform): Self::SystemData){
        let half_x = dim.width() / 2.0;
        let half_y = dim.height() / 2.0;
        if let Some((posx,posy)) = input.mouse_position(){
            let offset_x = half_x - posx as f32;
            let offset_y = half_y - posy as f32;
            for (_,transform) in (&camera,&mut transform).join(){

                transform.rotate_local(Vector3::new(1.0,0.0,0.0),Deg(offset_y * self.sensitivity_y));
                transform.rotate_global(Vector3::new(0.0,1.0,0.0),Deg(offset_x * self.sensitivity_x));
            }
        }
    }
}