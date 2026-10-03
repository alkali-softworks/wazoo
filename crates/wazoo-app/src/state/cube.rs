/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * 3D Bouncing Video Cube Screensaver State
 *
 * Manages the physics, multi-axis 3D rotations, edge collision detection,
 * and neon palette cycling for interactive floating video cubes.
 */

pub const NEON_PALETTE: &[[f32; 4]] = &[
    [0.26, 0.72, 0.51, 1.0], // Emerald Green (Wazoo signature)
    [0.00, 0.90, 1.00, 1.0], // Electric Cyan
    [0.83, 0.00, 0.98, 1.0], // Neon Violet
    [1.00, 0.15, 0.45, 1.0], // Hot Coral Pink
    [1.00, 0.84, 0.00, 1.0], // Solar Gold
    [1.00, 0.42, 0.00, 1.0], // Vivid Amber
    [0.16, 0.47, 1.00, 1.0], // Laser Blue
];

#[derive(Debug, Clone)]
pub struct BouncingCube {
    pub id: u64,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub size: f32,
    pub rx: f32,
    pub ry: f32,
    pub rz: f32,
    pub vrx: f32,
    pub vry: f32,
    pub vrz: f32,
    pub edge_color: [f32; 4],
    pub color_index: usize,
    pub player_index: usize,
    pub player_id: Option<wazoo_media::PlayerId>,
    pub bounce_count: u32,
}

impl BouncingCube {
    pub fn on_bounce(&mut self) {
        self.bounce_count = self.bounce_count.wrapping_add(1);
        self.color_index = (self.color_index + 1) % NEON_PALETTE.len();
        self.edge_color = NEON_PALETTE[self.color_index];

        // Maintain calm, smooth spin dynamics on bounce without chaotic fluctuations
        let speed_factor = if self.bounce_count % 2 == 0 { 1.02 } else { 0.98 };
        let sign = if self.vry < 0.0 { -1.0 } else { 1.0 };
        let abs_vry = (self.vry.abs() * speed_factor).clamp(0.005, 0.010);
        self.vry = abs_vry * sign;
        self.vrx = (self.vrx * speed_factor).clamp(0.0008, 0.0025);
        self.vrz = (self.vrz * speed_factor).clamp(0.0005, 0.0018);
    }
}

#[derive(Debug, Clone)]
pub struct CubeState {
    pub enabled: bool,
    pub desktop_overlay: bool,
    pub spawned_for_desktop: bool,
    pub cubes: Vec<BouncingCube>,
    pub next_id: u64,
    pub speed_multiplier: f32,
    pub size_multiplier: f32,
}

impl Default for CubeState {
    fn default() -> Self {
        Self {
            enabled: false,
            desktop_overlay: false,
            spawned_for_desktop: false,
            cubes: Vec::new(),
            next_id: 1,
            speed_multiplier: 1.0,
            size_multiplier: 1.0,
        }
    }
}

impl CubeState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if the 3D cube screensaver or overlay is active and at least one cube exists.
    pub fn is_present(&self) -> bool {
        self.enabled && !self.cubes.is_empty()
    }

    pub fn spawn_cube(&mut self, window_w: f32, window_h: f32, player_idx: usize) {
        self.spawn_cube_with_player(window_w, window_h, player_idx, None);
    }

    pub fn spawn_cube_with_player(
        &mut self,
        window_w: f32,
        window_h: f32,
        player_idx: usize,
        player_id: Option<wazoo_media::PlayerId>,
    ) {
        let w = window_w.max(300.0);
        let h = window_h.max(200.0);
        let count = self.cubes.len();

        // Stagger spawn positions slightly for each additional cube
        let offset_x = ((count * 47) % 180) as f32 - 90.0;
        let offset_y = ((count * 31) % 120) as f32 - 60.0;
        let center_x = (w * 0.5 + offset_x).clamp(100.0, w - 100.0);
        let center_y = (h * 0.5 + offset_y).clamp(100.0, h - 100.0);

        // Screensaver velocity trajectory
        let dir_x = if count % 2 == 0 { 1.0 } else { -1.0 };
        let dir_y = if (count / 2) % 2 == 0 { 1.0 } else { -1.0 };
        let vx = (2.8 + (count as f32 * 0.4) % 1.5) * dir_x;
        let vy = (2.2 + (count as f32 * 0.5) % 1.4) * dir_y;

        // Alternate spin directions for multiple cubes with a relaxed, majestic spin rate (~25°/s)
        let spin_dir = if count % 2 == 0 { 1.0 } else { -1.0 };
        let vry = (0.0072 + ((count % 3) as f32) * 0.0010) * spin_dir;
        let vrx = 0.0015;
        let vrz = 0.0010;

        let color_idx = count % NEON_PALETTE.len();
        let cube = BouncingCube {
            id: self.next_id,
            x: center_x,
            y: center_y,
            vx,
            vy,
            size: 95.0,
            // Forward showcase tilt (~22°) and subtle roll (~7°) so top and front faces are always clearly readable
            rx: 0.38 + ((count % 3) as f32) * 0.04,
            ry: (count as f32 * 0.8) % 3.14,
            rz: 0.12 * spin_dir,
            vrx,
            vry,
            vrz,
            edge_color: NEON_PALETTE[color_idx],
            color_index: color_idx,
            player_index: player_idx,
            player_id,
            bounce_count: 0,
        };

        self.next_id = self.next_id.wrapping_add(1);
        self.cubes.push(cube);
        self.enabled = true;
    }

    pub fn remove_cube(&mut self) -> Option<BouncingCube> {
        let removed = self.cubes.pop();
        if self.cubes.is_empty() {
            self.enabled = false;
            self.desktop_overlay = false;
        }
        removed
    }

    pub fn clear(&mut self) {
        self.cubes.clear();
        self.enabled = false;
        self.desktop_overlay = false;
        self.spawned_for_desktop = false;
    }

    pub fn toggle(&mut self, window_w: f32, window_h: f32, player_idx: usize) -> bool {
        self.toggle_with_player(window_w, window_h, player_idx, None)
    }

    pub fn toggle_with_player(
        &mut self,
        window_w: f32,
        window_h: f32,
        player_idx: usize,
        player_id: Option<wazoo_media::PlayerId>,
    ) -> bool {
        if self.is_present() {
            self.clear();
            false
        } else {
            self.spawn_cube_with_player(window_w, window_h, player_idx, player_id);
            true
        }
    }

    pub fn tick(&mut self, window_w: f32, window_h: f32) {
        if !self.is_present() {
            return;
        }

        let w = window_w.max(200.0);
        let h = window_h.max(150.0);
        let sm = self.speed_multiplier.clamp(0.2, 4.0);
        let szm = self.size_multiplier.clamp(0.4, 3.0);

        for cube in &mut self.cubes {
            let r = (cube.size * szm).max(20.0);

            cube.x += cube.vx * sm;
            cube.y += cube.vy * sm;

            // Smooth steady spin around Y axis
            cube.ry += cube.vry * sm;

            // Gentle pitch sway bounded within an optimal viewing window [14°..30°] (0.24 to 0.52 rad)
            // Keeps the top face in view and video right-side up without chaotic somersaults
            cube.rx += cube.vrx * sm;
            if cube.rx < 0.24 && cube.vrx < 0.0 {
                cube.rx = 0.24;
                cube.vrx = cube.vrx.abs();
            } else if cube.rx > 0.52 && cube.vrx > 0.0 {
                cube.rx = 0.52;
                cube.vrx = -cube.vrx.abs();
            }

            // Gentle subtle roll sway bounded within [-14°..14°] (-0.24 to 0.24 rad)
            cube.rz += cube.vrz * sm;
            if cube.rz < -0.24 && cube.vrz < 0.0 {
                cube.rz = -0.24;
                cube.vrz = cube.vrz.abs();
            } else if cube.rz > 0.24 && cube.vrz > 0.0 {
                cube.rz = 0.24;
                cube.vrz = -cube.vrz.abs();
            }

            // X-axis collision (left/right walls)
            if cube.x - r <= 0.0 {
                cube.x = r;
                cube.vx = cube.vx.abs();
                cube.on_bounce();
            } else if cube.x + r >= w {
                cube.x = (w - r).max(r);
                cube.vx = -cube.vx.abs();
                cube.on_bounce();
            }

            // Y-axis collision (top/bottom walls)
            if cube.y - r <= 0.0 {
                cube.y = r;
                cube.vy = cube.vy.abs();
                cube.on_bounce();
            } else if cube.y + r >= h {
                cube.y = (h - r).max(r);
                cube.vy = -cube.vy.abs();
                cube.on_bounce();
            }
        }
    }
}
