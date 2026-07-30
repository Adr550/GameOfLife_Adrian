use raylib::prelude::*;
use std::time::{Duration, Instant};
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;

// Configuración
const UI_HEIGHT: i32 = 80;
const MIN_WINDOW_SIZE: i32 = 400;
const MAX_WINDOW_SIZE: i32 = 1600;
const DEFAULT_WINDOW_SIZE: i32 = 900;
const MIN_GRID_SIZE: usize = 30;
const MAX_GRID_SIZE: usize = 300;
const DEFAULT_GRID_SIZE: usize = 150;

// Estructura principal del juego
pub struct GameOfLife {
    cells: Vec<u8>,
    next_cells: Vec<u8>,
    width: usize,
    height: usize,
    generation: u64,
    population: usize,
    paused: bool,
    speed: f32,
    previous_population: usize,
    stable_count: u32,
    max_generation: u64,
    seed: u64,
    window_size: i32,
    grid_size: usize,
    fps_counter: u32,
    last_fps_update: Instant,
}

// Estructura para patrones
pub struct Pattern {
    pub name: String,
    pub width: usize,
    pub height: usize,
    pub cells: Vec<u8>,
}

impl Pattern {
    pub fn new(name: &str, width: usize, height: usize, cells: &[u8]) -> Self {
        Self {
            name: name.to_string(),
            width,
            height,
            cells: cells.to_vec(),
        }
    }
}

impl GameOfLife {
    pub fn new(window_size: i32, grid_size: usize) -> Self {
        let size = grid_size * grid_size;
        Self {
            cells: vec![0; size],
            next_cells: vec![0; size],
            width: grid_size,
            height: grid_size,
            generation: 0,
            population: 0,
            paused: true,
            speed: 0.5,
            previous_population: 0,
            stable_count: 0,
            max_generation: 0,
            seed: 42,
            window_size,
            grid_size,
            fps_counter: 0,
            last_fps_update: Instant::now(),
        }
    }

    pub fn resize(&mut self, new_window_size: i32, new_grid_size: usize) {
        self.window_size = new_window_size.clamp(MIN_WINDOW_SIZE, MAX_WINDOW_SIZE);
        self.grid_size = new_grid_size.clamp(MIN_GRID_SIZE, MAX_GRID_SIZE);
        
        let new_size = self.grid_size * self.grid_size;
        let mut new_cells = vec![0; new_size];
        let new_next_cells = vec![0; new_size];
        
        let copy_width = self.width.min(self.grid_size);
        let copy_height = self.height.min(self.grid_size);
        
        for y in 0..copy_height {
            for x in 0..copy_width {
                let old_idx = y * self.width + x;
                let new_idx = y * self.grid_size + x;
                new_cells[new_idx] = self.cells[old_idx];
            }
        }
        
        self.width = self.grid_size;
        self.height = self.grid_size;
        self.cells = new_cells;
        self.next_cells = new_next_cells;
        self.update_population();
    }

    pub fn randomize_with_seed(&mut self, seed: u64, density: f32) {
        self.seed = seed;
        let mut rng = StdRng::seed_from_u64(seed);
        for cell in self.cells.iter_mut() {
            *cell = if rng.gen_bool(density as f64) { 1 } else { 0 };
        }
        self.generation = 0;
        self.stable_count = 0;
        self.max_generation = 0;
        self.update_population();
    }

    pub fn randomize(&mut self) {
        let mut rng = rand::thread_rng();
        let seed = rng.r#gen::<u64>();
        self.randomize_with_seed(seed, 0.25);
    }

    pub fn clear(&mut self) {
        self.cells.fill(0);
        self.generation = 0;
        self.population = 0;
        self.stable_count = 0;
        self.max_generation = 0;
    }

    fn update_population(&mut self) {
        self.population = self.cells.iter().filter(|&&c| c == 1).count();
    }

    pub fn count_neighbors(&self, x: usize, y: usize) -> u8 {
        let mut count = 0;
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = ((x as isize + dx + self.width as isize) % self.width as isize) as usize;
                let ny = ((y as isize + dy + self.height as isize) % self.height as isize) as usize;
                count += self.cells[ny * self.width + nx];
            }
        }
        count
    }

    pub fn update(&mut self) {
        self.previous_population = self.population;
        self.population = 0;
        
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                let neighbors = self.count_neighbors(x, y);
                let alive = self.cells[idx];
                
                if alive == 1 {
                    if neighbors < 2 || neighbors > 3 {
                        self.next_cells[idx] = 0;
                    } else {
                        self.next_cells[idx] = 1;
                        self.population += 1;
                    }
                } else {
                    if neighbors == 3 {
                        self.next_cells[idx] = 1;
                        self.population += 1;
                    } else {
                        self.next_cells[idx] = 0;
                    }
                }
            }
        }
        
        std::mem::swap(&mut self.cells, &mut self.next_cells);
        self.generation += 1;
        
        if self.generation > self.max_generation {
            self.max_generation = self.generation;
        }
        
        if self.population == self.previous_population && self.population > 0 {
            self.stable_count += 1;
        } else {
            self.stable_count = 0;
        }
    }

    pub fn load_pattern(&mut self, pattern: &Pattern, x: isize, y: isize) {
        self.clear();
        
        for dy in 0..pattern.height {
            for dx in 0..pattern.width {
                let grid_x = x + dx as isize;
                let grid_y = y + dy as isize;
                
                if grid_x >= 0 && grid_x < self.width as isize && 
                   grid_y >= 0 && grid_y < self.height as isize {
                    let idx = (grid_y as usize) * self.width + (grid_x as usize);
                    self.cells[idx] = pattern.cells[dy * pattern.width + dx];
                }
            }
        }
        self.update_population();
        self.stable_count = 0;
        self.max_generation = 0;
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle) {
        let cell_size = self.window_size / self.grid_size as i32;
        let offset_x = (self.window_size - (self.grid_size as i32 * cell_size)) / 2;
        let offset_y = (self.window_size - (self.grid_size as i32 * cell_size)) / 2;
        
        let mut next_state = vec![0; self.cells.len()];
        let mut will_die_next = vec![false; self.cells.len()];
        
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                let neighbors = self.count_neighbors(x, y);
                let alive = self.cells[idx];
                
                if alive == 1 {
                    if neighbors < 2 || neighbors > 3 {
                        next_state[idx] = 0;
                        will_die_next[idx] = true;
                    } else {
                        next_state[idx] = 1;
                    }
                } else {
                    if neighbors == 3 {
                        next_state[idx] = 1;
                    } else {
                        next_state[idx] = 0;
                    }
                }
            }
        }
        
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                let color;
                
                if self.cells[idx] == 1 {
                    if will_die_next[idx] {
                        color = Color::new(255, 50, 50, 255);
                    } else {
                        color = Color::new(50, 255, 50, 255);
                    }
                } else {
                    if next_state[idx] == 1 {
                        color = Color::new(50, 200, 255, 255);
                    } else {
                        color = Color::new(5, 5, 10, 255);
                    }
                }
                
                let pos_x = offset_x + (x as i32 * cell_size);
                let pos_y = offset_y + (y as i32 * cell_size);
                
                if cell_size > 2 {
                    d.draw_rectangle(pos_x, pos_y, cell_size - 1, cell_size - 1, color);
                } else {
                    d.draw_rectangle(pos_x, pos_y, cell_size, cell_size, color);
                }
            }
        }
    }

    pub fn draw_ui(&mut self, d: &mut RaylibDrawHandle) {
        let ui_y = self.window_size;
        
        d.draw_rectangle(0, ui_y, self.window_size + 200, UI_HEIGHT, Color::new(20, 20, 30, 255));
        
        self.fps_counter += 1;
        if self.last_fps_update.elapsed() >= Duration::from_secs(1) {
            self.fps_counter = 0;
            self.last_fps_update = Instant::now();
        }
        
        let status = if self.paused { "⏸ PAUSADO" } else { "▶ EJECUTANDO" };
        
        let info = format!(
            "Ronda: {} | Población: {} | {} | Vel: {:.1}x | Estable: {} | Seed: {}",
            self.generation,
            self.population,
            status,
            self.speed,
            self.stable_count,
            self.seed
        );
        d.draw_text(&info, 10, ui_y + 8, 16, Color::LIGHTGRAY);
        
        let info2 = format!(
            "Máx Gen: {} | Grid: {}x{} | Ventana: {}x{} | FPS: {}",
            self.max_generation,
            self.grid_size,
            self.grid_size,
            self.window_size,
            self.window_size,
            self.fps_counter
        );
        d.draw_text(&info2, 10, ui_y + 30, 14, Color::GRAY);
        
        d.draw_text("🟢 Sobrevive  🔴 Muere  🔵 Nace  ⬛ Muerta", 10, ui_y + 50, 13, Color::LIGHTGRAY);
        
        d.draw_text("[Espacio] Pausa  [R] Random  [C] Clear  [+/-] Vel  [1-0] Patrones  [S] Seed  [F] Fullscreen", 
                   self.window_size - 500, ui_y + 8, 13, Color::LIGHTGRAY);
        
        d.draw_text("[Q/A] Grid +/-  [W/S] Window +/-", 
                   self.window_size - 400, ui_y + 30, 13, Color::LIGHTGRAY);
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn change_speed(&mut self, delta: f32) {
        self.speed = (self.speed + delta).clamp(0.1, 3.0);
    }

    pub fn get_seed(&self) -> u64 {
        self.seed
    }

    pub fn get_window_size(&self) -> i32 {
        self.window_size
    }

    pub fn get_grid_size(&self) -> usize {
        self.grid_size
    }
}

// Definir patrones
fn create_patterns() -> Vec<Pattern> {
    // Puertas lógicas
    let and_gate = vec![
        0,0,0,0,0,0,0,0,0,
        0,1,0,0,0,0,0,1,0,
        0,0,0,0,0,0,0,0,0,
        0,0,0,1,1,1,0,0,0,
        0,0,0,1,0,1,0,0,0,
        0,0,0,1,1,1,0,0,0,
        0,0,0,0,0,0,0,0,0,
        0,0,0,0,1,0,0,0,0,
        0,0,0,0,0,0,0,0,0
    ];

    let or_gate = vec![
        0,0,0,0,0,0,0,0,0,
        0,1,0,0,0,0,0,1,0,
        0,0,0,0,0,0,0,0,0,
        0,0,0,1,1,1,0,0,0,
        0,0,0,1,0,1,0,0,0,
        0,0,0,1,1,1,0,0,0,
        0,0,0,1,1,1,0,0,0,
        0,0,0,1,0,1,0,0,0,
        0,0,0,0,0,0,0,0,0
    ];

    let not_gate = vec![
        0,0,0,0,0,0,0,
        0,1,0,0,0,1,0,
        0,0,0,0,0,0,0,
        0,0,0,1,0,0,0,
        0,0,0,0,0,0,0,
        0,0,1,0,0,1,0,
        0,0,0,0,0,0,0
    ];

    let xor_gate = vec![
        0,0,0,0,0,0,0,0,0,
        0,1,0,0,0,0,0,1,0,
        0,0,0,0,0,0,0,0,0,
        0,0,0,1,1,1,0,0,0,
        0,0,0,1,0,1,0,0,0,
        0,0,0,1,1,1,0,0,0,
        0,0,0,0,0,0,0,0,0,
        0,0,0,0,1,0,0,0,0,
        0,0,0,0,0,0,0,0,0
    ];

    // Patrones clásicos
    let glider = vec![0,1,0, 0,0,1, 1,1,1];
    let blinker = vec![0,1,0, 0,1,0, 0,1,0];
    let block = vec![1,1, 1,1];
    let beehive = vec![0,1,1,0, 1,0,0,1, 0,1,1,0];
    let toad = vec![0,1,1,1, 1,1,1,0];
    let beacon = vec![1,1,0,0, 1,1,0,0, 0,0,1,1, 0,0,1,1];
    let pulsar = vec![
        0,0,1,1,1,0,0,0,1,1,1,0,0,
        0,0,0,0,0,0,0,0,0,0,0,0,0,
        1,0,0,0,0,1,0,1,0,0,0,0,1,
        1,0,0,0,0,1,0,1,0,0,0,0,1,
        1,0,0,0,0,1,0,1,0,0,0,0,1,
        0,0,1,1,1,0,0,0,1,1,1,0,0,
        0,0,0,0,0,0,0,0,0,0,0,0,0,
        0,0,1,1,1,0,0,0,1,1,1,0,0,
        1,0,0,0,0,1,0,1,0,0,0,0,1,
        1,0,0,0,0,1,0,1,0,0,0,0,1,
        1,0,0,0,0,1,0,1,0,0,0,0,1,
        0,0,0,0,0,0,0,0,0,0,0,0,0,
        0,0,1,1,1,0,0,0,1,1,1,0,0
    ];
    let lwss = vec![
        0,1,0,0,1,
        1,0,0,0,0,
        1,0,0,0,1,
        1,1,1,1,0
    ];

    // PATRONES LONGEVOS
    let r_pentomino = vec![
        0,1,1,
        1,1,0,
        0,1,0
    ];

    let diehard = vec![
        0,0,0,0,0,0,1,0,
        1,1,0,0,0,0,0,0,
        0,1,0,0,0,1,1,1
    ];

    let acorn = vec![
        0,1,0,0,0,0,0,
        0,0,0,1,0,0,0,
        1,1,0,0,1,1,1
    ];

    vec![
        Pattern::new("AND Gate", 9, 9, &and_gate),
        Pattern::new("OR Gate", 9, 9, &or_gate),
        Pattern::new("NOT Gate", 7, 7, &not_gate),
        Pattern::new("XOR Gate", 9, 9, &xor_gate),
        Pattern::new("Glider", 3, 3, &glider),
        Pattern::new("Blinker", 3, 3, &blinker),
        Pattern::new("Block", 2, 2, &block),
        Pattern::new("Beehive", 4, 3, &beehive),
        Pattern::new("Toad", 4, 2, &toad),
        Pattern::new("Beacon", 4, 4, &beacon),
        Pattern::new("Pulsar", 13, 13, &pulsar),
        Pattern::new("LWSS", 5, 4, &lwss),
        Pattern::new("R-Pentomino", 3, 3, &r_pentomino),
        Pattern::new("Diehard", 8, 3, &diehard),
        Pattern::new("Acorn", 7, 3, &acorn),
    ]
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(DEFAULT_WINDOW_SIZE, DEFAULT_WINDOW_SIZE + UI_HEIGHT)
        .title("Game of Life - Con Controles de Tamaño")
        .build();

    let mut game = GameOfLife::new(DEFAULT_WINDOW_SIZE, DEFAULT_GRID_SIZE);
    game.randomize_with_seed(42, 0.25);
    game.paused = false;
    
    let patterns = create_patterns();
    let mut last_update = Instant::now();
    let mut seed_counter = 0;
    let mut is_fullscreen = false;
    let mut window_size = DEFAULT_WINDOW_SIZE;

    while !rl.window_should_close() {
        // Input
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            game.toggle_pause();
        }
        
        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            game.randomize();
        }
        
        if rl.is_key_pressed(KeyboardKey::KEY_C) {
            game.clear();
        }
        
        if rl.is_key_pressed(KeyboardKey::KEY_S) {
            seed_counter += 1;
            game.randomize_with_seed(seed_counter, 0.25);
        }
        
        // Cambiar tamaño del grid (Q/A)
        if rl.is_key_pressed(KeyboardKey::KEY_Q) {
            let new_grid = (game.get_grid_size() as i32 + 10).min(MAX_GRID_SIZE as i32) as usize;
            game.resize(game.get_window_size(), new_grid);
        }
        if rl.is_key_pressed(KeyboardKey::KEY_A) {
            let new_grid = (game.get_grid_size() as i32 - 10).max(MIN_GRID_SIZE as i32) as usize;
            game.resize(game.get_window_size(), new_grid);
        }
        
        // Cambiar tamaño de la ventana (W/S)
        if rl.is_key_pressed(KeyboardKey::KEY_W) {
            let new_size = (game.get_window_size() + 50).min(MAX_WINDOW_SIZE);
            game.resize(new_size, game.get_grid_size());
            rl.set_window_size(new_size, new_size + UI_HEIGHT);
            window_size = new_size;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_S) {
            let new_size = (game.get_window_size() - 50).max(MIN_WINDOW_SIZE);
            game.resize(new_size, game.get_grid_size());
            rl.set_window_size(new_size, new_size + UI_HEIGHT);
            window_size = new_size;
        }
        
        // Fullscreen
        if rl.is_key_pressed(KeyboardKey::KEY_F) {
            is_fullscreen = !is_fullscreen;
            rl.toggle_fullscreen();
            if !is_fullscreen {
                rl.set_window_size(window_size, window_size + UI_HEIGHT);
            }
        }
        
        // Velocidad
        if rl.is_key_pressed(KeyboardKey::KEY_EQUAL) || rl.is_key_pressed(KeyboardKey::KEY_KP_ADD) {
            game.change_speed(0.1);
        }
        if rl.is_key_pressed(KeyboardKey::KEY_MINUS) || rl.is_key_pressed(KeyboardKey::KEY_KP_SUBTRACT) {
            game.change_speed(-0.1);
        }
        
        // Cargar patrones (teclas 1-9 y 0)
        let key_map = [
            KeyboardKey::KEY_ONE,
            KeyboardKey::KEY_TWO,
            KeyboardKey::KEY_THREE,
            KeyboardKey::KEY_FOUR,
            KeyboardKey::KEY_FIVE,
            KeyboardKey::KEY_SIX,
            KeyboardKey::KEY_SEVEN,
            KeyboardKey::KEY_EIGHT,
            KeyboardKey::KEY_NINE,
            KeyboardKey::KEY_ZERO,
        ];
        
        for i in 0..patterns.len().min(10) {
            if rl.is_key_pressed(key_map[i]) {
                let grid_x = (game.width as isize / 2) - (patterns[i].width as isize / 2);
                let grid_y = (game.height as isize / 2) - (patterns[i].height as isize / 2);
                game.load_pattern(&patterns[i], grid_x, grid_y);
                game.paused = false;
            }
        }

        // Actualizar
        if !game.paused {
            let base_interval = 200;
            let update_interval = Duration::from_millis((base_interval as f32 / game.speed) as u64);
            if last_update.elapsed() >= update_interval {
                game.update();
                last_update = Instant::now();
            }
        }

        // Dibujar
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        
        game.draw(&mut d);
        game.draw_ui(&mut d);
    }
}