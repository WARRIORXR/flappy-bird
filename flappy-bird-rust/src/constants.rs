pub const GAME_WIDTH: f32 = 400.0;
pub const GAME_HEIGHT: f32 = 700.0;

// Bird physics
pub const GRAVITY: f32 = 0.55;
pub const FLAP_STRENGTH: f32 = -8.5;
pub const MAX_FALL_SPEED: f32 = 12.0;

// Bird dimensions & position
pub const BIRD_X: f32 = 80.0;
pub const BIRD_WIDTH: f32 = 38.0;
pub const BIRD_HEIGHT: f32 = 28.0;

// Pipe settings
pub const PIPE_WIDTH: f32 = 60.0;
pub const PIPE_GAP: f32 = 150.0;
pub const PIPE_SPACING: f32 = 220.0;
pub const PIPE_SPEED: f32 = 2.5;
pub const PIPE_SPEED_INC: f32 = 0.08;
pub const PIPE_MAX_SPEED: f32 = 6.0;

// Ground
pub const GROUND_HEIGHT: f32 = 80.0;
pub const GROUND_SPEED_MULT: f32 = 1.0;

// Clouds
pub const NUM_CLOUDS: usize = 6;
pub const CLOUD_MIN_SPEED: f32 = 0.3;
pub const CLOUD_MAX_SPEED: f32 = 0.8;

// Wing animation
pub const WING_FLAP_SPEED: f32 = 0.15;

// Rotation limits
pub const MAX_UP_ROTATION: f32 = -0.5;
pub const MAX_DOWN_ROTATION: f32 = 1.2;
