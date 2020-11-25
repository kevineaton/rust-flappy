use bracket_lib::prelude::*;

// set up some constants
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const FRAME_DURATION: f32 = 75.0;

// handle the game state
enum GameMode {
    Menu,
    Playing,
    Paused,
    End,
}

// store the global state
struct State {
    mode: GameMode,
    player: Player,
    frame_time: f32,
    obstacle: Obstacle,
    score: i32,
}

impl State {
    fn new() -> Self {
        State {
            mode: GameMode::Menu,
            player: Player::new(5,25),
            frame_time: 0.0,
            obstacle: Obstacle::new(SCREEN_WIDTH, 0),
            score: 0,
        }
    }

    // the actual game loop
    fn play(&mut self, ctx: &mut BTerm) {
        ctx.cls_bg(NAVY);
        self.frame_time += ctx.frame_time_ms;
        if self.frame_time > FRAME_DURATION {
            self.frame_time = 0.0;
            self.player.gravity_and_move()
        }
        if let Some(VirtualKeyCode::Space) = ctx.key {
            self.player.flap();
        }
        if let Some(VirtualKeyCode::Escape) = ctx.key {
            self.mode = GameMode::Paused;
        }
        self.player.render(ctx);
        ctx.print(0, 0, "Press SPACE to flap");
        ctx.print(0, 1, &format!("Score: {}", self.score));

        // set up the obstacles
        self.obstacle.render(ctx, self.player.x);
        if self.player.x > self.obstacle.x {
            self.score += 1;
            self.obstacle = Obstacle::new(self.player.x + SCREEN_WIDTH, self.score);
        }

        if self.player.y > SCREEN_HEIGHT || self.obstacle.hit_obstacle(&self.player) {
            self.mode = GameMode::End;
        }
    }

    fn main_menu(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "Welcome to the game!");
        ctx.print_centered(8, "(P) Play");
        ctx.print_centered(9, "(Q) Quit");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }

    fn restart(&mut self) {
        self.player = Player::new(5, 25);
        self.frame_time = 0.0;
        self.mode = GameMode::Playing;
    }

    fn paused(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "PAUSED");
        ctx.print_centered(7, &format!("Score: {}", self.score));
        ctx.print_centered(9, "(P) Play");
        ctx.print_centered(11, "(Q) Quit");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.mode = GameMode::Playing,
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }

    fn dead(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "You lost!");
        ctx.print_centered(7, &format!("Score: {}", self.score));
        ctx.print_centered(9, "(P) Play");
        ctx.print_centered(11, "(Q) Quit");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }
}

// use the GameState trait from the library
impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        match self.mode {
            GameMode::Menu => self.main_menu(ctx),
            GameMode::End => self.dead(ctx),
            GameMode::Playing => self.play(ctx),
            GameMode::Paused => self.paused(ctx),
        }
    }
}

// the player
struct Player {
    x: i32, // world space position, since player is always on the left
    y: i32, // screen space vertical
    velocity: f32,
}

impl Player {
    fn new(x: i32, y: i32) -> Self {
        Player {
            x,
            y,
            velocity: 0.0,
        }
    }

    fn render(&mut self, ctx: &mut BTerm) {
        ctx.set(
            0,
            self.y,
            RGB::named(YELLOW),
            RGB::named(BLACK),
            to_cp437('@'),
        );
    }

    fn gravity_and_move(&mut self) {
        if self.velocity < 2.0 {
            self.velocity += 0.2;
        }
        self.y += self.velocity as i32;
        self.x += 1;
        if self.y < 0 {
            self.y = 0;
        }
    }

    fn flap(&mut self) {
        self.velocity = -2.0;
    }
}

// obstacles have gaps and a size that will vary
struct Obstacle {
    x: i32,
    gap_y: i32,
    size: i32,
}

impl Obstacle {
    fn new(x: i32, score: i32) -> Self {
        let mut random = RandomNumberGenerator::new();
        Obstacle {
            x,
            gap_y: random.range(10, 40),
            size: i32::max(2, 20 - score),
        }
    }

    fn render(&mut self, ctx: &mut BTerm, player_x: i32) {
        let screen_x = self.x - player_x;
        let half_size = self.size / 2;

        // top half
        for y in 0..self.gap_y - half_size {
            ctx.set(
               screen_x,
               y,
               RGB::named(RED),
               RGB::named(BLACK),
               to_cp437('|'), 
            );
        }
        // bottom half
        for y in self.gap_y + half_size..SCREEN_HEIGHT {
            ctx.set(
                screen_x,
                y,
                RGB::named(RED),
                RGB::named(BLACK),
                to_cp437('|'),
            );
        }
    }

    fn hit_obstacle(&self, player: &Player) -> bool {
        let half_size = self.size / 2;
        let does_x_match = player.x == self.x;
        let player_above_gap = player.y < self.gap_y - half_size;
        let player_below_gap = player.y > self.gap_y + half_size;
        does_x_match && (player_above_gap || player_below_gap)
    }
}

fn main() -> BError {
    // build and run, returning the Result
    let context = BTermBuilder::simple80x50()
        .with_title("Flappy Rust")
        .build()?;
    main_loop(context, State::new())
}
