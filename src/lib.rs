use std::{
    fs,
    io::{self, Read, Seek},
};

#[derive(Clone, Copy, Debug)]
pub enum MapLumpIndex {
    Things = 1,
    LineDefs = 2,
    SideDefs = 3,
    Vertexes = 4,
    Segs = 5,
    SSectors = 6,
    Nodes = 7,
    Sectors = 8,
    Reject = 9,
    BlockMap = 10,
}

#[derive(Clone, Copy, Debug)]
pub enum LineDefFlags {
    Blocking = 0,
    BlockMonsters = 1,
    TwoSided = 2,
    DontPegTop = 4,
    DontPegBottom = 8,
    Secret = 16,
    SoundBlock = 32,
    DontDraw = 64,
    Draw = 128,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Thing {
    pub x: i16,
    pub y: i16,
    pub angle: i16,
    pub t_type: i16,
    pub flags: i16,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct LineDef {
    pub start_vertex: i16,
    pub end_vertex: i16,
    pub flags: i16,
    pub special_type: i16,
    pub sector_tag: i16,
    pub right_sidedef: i16,
    pub left_sidedef: i16,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SideDef {
    pub x_offset: i16,
    pub y_offset: i16,
    pub upper_texture: [u8; 8],
    pub lower_texture: [u8; 8],
    pub middle_texture: [u8; 8],
    pub sector: i16,
}

impl SideDef {
    pub fn upper_texture(&self) -> String {
        WAD::slice_to_string(&self.upper_texture)
    }

    pub fn lower_texture(&self) -> String {
        WAD::slice_to_string(&self.lower_texture)
    }

    pub fn middle_texture(&self) -> String {
        WAD::slice_to_string(&self.middle_texture)
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Vertex {
    pub x: i16,
    pub y: i16,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Seg {
    pub start_vertex: i16,
    pub end_vertex: i16,
    pub angle: i16,
    pub linedef: i16,
    pub direction: i16,
    pub offset: i16,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SubSector {
    pub num_segs: i16,
    pub first_seg: i16,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Node {
    pub x_partition: i16,
    pub y_partition: i16,
    pub change_x_partition: i16,
    pub change_y_partition: i16,
    pub right_bbox: [i16; 4],
    pub left_bbox: [i16; 4],
    pub right_child: i16,
    pub left_child: i16,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Sector {
    pub floor_height: i16,
    pub ceiling_height: i16,
    pub floor_texture: [u8; 8],
    pub ceiling_texture: [u8; 8],
    pub light_level: i16,
    pub special_type: i16,
    pub tag: i16,
}

impl Sector {
    pub fn floor_texture(&self) -> String {
        WAD::slice_to_string(&self.floor_texture)
    }

    pub fn ceiling_texture(&self) -> String {
        WAD::slice_to_string(&self.ceiling_texture)
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Reject {
    pub num_rejects: i16,
    pub first_reject: i16,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct BlockMap {
    pub x_origin: i16,
    pub y_origin: i16,
    pub columns: i16,
    pub rows: i16,
    pub offsets: [i16; 1],
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Directory {
    pub offset: u32,
    pub size: u32,
    pub name: [u8; 8],
}

impl Directory {
    pub fn name(&self) -> String {
        WAD::slice_to_string(&self.name)
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Header {
    pub identifier: [u8; 4],
    pub count: u32,
    pub offset: u32,
}

impl Default for Header {
    fn default() -> Self {
        Self {
            identifier: [0; 4],
            count: 0,
            offset: 0,
        }
    }
}

impl Header {
    pub fn identification(&self) -> String {
        WAD::slice_to_string(&self.identifier)
    }
}

#[derive(Debug)]
pub struct WAD {
    pub things: Vec<Thing>,
    pub line_defs: Vec<LineDef>,
    pub side_defs: Vec<SideDef>,
    pub vertexes: Vec<Vertex>,
    pub segs: Vec<Seg>,
    pub ssectors: Vec<SubSector>,
    pub nodes: Vec<Node>,
    pub sectors: Vec<Sector>,

    pub directory: Vec<Directory>,
    pub header: Header,

    map_index: Option<usize>,
    file: fs::File,
}

impl WAD {
    pub fn slice_to_string(slice: &[u8]) -> String {
        slice
            .iter()
            .filter(|&&c| c != 0)
            .map(|&c| c as char)
            .collect::<String>()
    }
}

impl WAD {
    const DIRECTORY_SIZE: usize = 16;
    const HEADER_SIZE: usize = 12;

    fn read_directory(&mut self) -> io::Result<()> {
        let offset = self.header.offset as u64;

        self.directory.clear();
        self.file.seek(io::SeekFrom::Start(offset))?;

        for _ in 0..(self.header.count as usize) {
            let mut bytes = [0; Self::DIRECTORY_SIZE];

            self.file.read_exact(&mut bytes)?;
            self.directory
                .push(unsafe { std::mem::transmute::<_, Directory>(bytes) });
        }

        Ok(())
    }

    fn read_header(&mut self) -> io::Result<()> {
        let mut bytes = [0; Self::HEADER_SIZE];

        self.file.seek(io::SeekFrom::Start(0))?;
        self.file.read_exact(&mut bytes)?;

        self.header = unsafe { std::mem::transmute::<_, Header>(bytes) };

        Ok(())
    }
}

impl WAD {
    // `offset` - Map index + MapLumpIndex.
    fn read_map_lump(&mut self, offset: usize) -> io::Result<Vec<u8>> {
        let lump = self.directory[offset];

        let mut bytes = vec![0; lump.size as usize];

        self.file.seek(io::SeekFrom::Start(lump.offset as u64))?;
        self.file.read_exact(&mut bytes)?;

        Ok(bytes)
    }

    fn read_map_lump_as<T>(&mut self, index: MapLumpIndex) -> io::Result<Vec<T>> {
        match self.map_index {
            Some(map_index) => {
                let bytes = self.read_map_lump(map_index + index as usize)?;

                let len = bytes.len() / std::mem::size_of::<T>();
                let ptr = bytes.as_ptr();

                let data = unsafe { Vec::from_raw_parts(ptr as *mut T, len, len) };
                std::mem::forget(bytes);

                Ok(data)
            }
            None => Err(io::Error::new(
                io::ErrorKind::Other,
                "No map has been loaded.",
            )),
        }
    }
}

impl WAD {
    pub fn change_map(&mut self, name: &str) -> Result<bool, io::Error> {
        for (i, dir) in self.directory.iter().enumerate() {
            if dir.name() == name {
                self.map_index = Some(i);

                self.things = self.read_map_lump_as(MapLumpIndex::Things)?;
                self.line_defs = self.read_map_lump_as(MapLumpIndex::LineDefs)?;
                self.side_defs = self.read_map_lump_as(MapLumpIndex::SideDefs)?;
                self.vertexes = self.read_map_lump_as(MapLumpIndex::Vertexes)?;
                self.segs = self.read_map_lump_as(MapLumpIndex::Segs)?;
                self.ssectors = self.read_map_lump_as(MapLumpIndex::SSectors)?;
                self.nodes = self.read_map_lump_as(MapLumpIndex::Nodes)?;
                self.sectors = self.read_map_lump_as(MapLumpIndex::Sectors)?;

                return Ok(true);
            }
        }

        Ok(false)
    }
}

impl WAD {
    pub fn setup(&mut self) -> io::Result<()> {
        self.read_header()?;
        self.read_directory()?;

        Ok(())
    }

    pub fn open(&mut self, path: &str) -> io::Result<()> {
        self.file = fs::File::open(path)?;
        self.setup()?;

        Ok(())
    }
}

impl WAD {
    pub fn new(path: &str) -> Result<Self, io::Error> {
        let mut ctx = Self {
            things: Vec::new(),
            line_defs: Vec::new(),
            side_defs: Vec::new(),
            vertexes: Vec::new(),
            segs: Vec::new(),
            ssectors: Vec::new(),
            nodes: Vec::new(),
            sectors: Vec::new(),

            directory: Vec::new(),
            header: Header::default(),

            map_index: None,
            file: fs::File::open(path)?,
        };

        ctx.setup()?;

        Ok(ctx)
    }
}

// - - -
#[derive(Debug)]
pub struct Player {
    pub thing: Thing,
    pub position: (f32, f32),
    pub angle: f32,
}

impl Player {
    pub fn new(thing: Thing) -> Self {
        Self {
            thing,
            position: (thing.x as f32, thing.y as f32),
            angle: thing.angle as f32,
        }
    }
}

// - - -
pub struct BSP <'a> {
    pub map_data: &'a WAD, 
    pub root_node_id: usize,
}

impl <'a> BSP <'a> {
    pub fn is_on_back_side(&self, player: &Player, node: &Node) -> bool {
        let dx = player.position.0 - node.x_partition as f32;
        let dy = player.position.1 - node.y_partition as f32;

        dx * node.change_y_partition as f32 - dy * node.change_x_partition as f32 <= 0.0
    }

    pub fn render_sub_sector(&self, player: &Player, sub_sector_id: u16) {
        let sub_sector = self.map_data.ssectors[sub_sector_id as usize];

        for i in 0..sub_sector.num_segs {
            let seg = &self.map_data.segs[sub_sector.first_seg as usize + i as usize];


        }
    }

    pub fn render_bsp_node(&self, player: &Player, node_id: u16) {
        let sub_sector_identifier = 0x8000;
        let mut sub_sector_id = 0;

        let node = &self.map_data.nodes[node_id as usize];

        if node_id >= sub_sector_identifier {
            sub_sector_id = node_id - sub_sector_identifier;

            self.render_sub_sector(player, sub_sector_id);            
            return
        }

        if self.is_on_back_side(player, node) {
            self.render_bsp_node(player, node.right_child as u16);
            self.render_bsp_node(player, node.left_child as u16);
        } else {
            self.render_bsp_node(player, node.left_child as u16);
            self.render_bsp_node(player, node.right_child as u16);
        }

            
    }

    pub fn update(&self, player: &Player) {
        self.render_bsp_node(player, self.root_node_id as u16); 
    }
}

impl <'a> BSP <'a> {
    pub fn new(map_data: &'a WAD) -> Self {
        let root_node_id = map_data.nodes.len() - 1;

        Self {
            map_data,
            root_node_id,
        }
    }
}

// - - -
pub struct Engine <'a> {
    pub window: &'a mut RenderWindow,
    pub wad: &'a WAD,
    pub player: Player,
    pub bsp: BSP<'a>,
}

impl <'a> Engine <'a> {
    pub fn draw_circle(&mut self, x: f32, y: f32, radius: f32, color: Color) {
        let mut circle = CircleShape::new(radius, 30);
        circle.set_fill_color(color);
        circle.set_position(Vector2f::new(x, y));

        self.window.draw(&circle);
    }

    pub fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: Color) {
        let mut line = RectangleShape::new();
        line.set_fill_color(color);
        line.set_size(Vector2f::new(1.0, 1.0));
        line.set_position(Vector2f::new(x1, y1));
        line.set_rotation((y2 - y1).atan2(x2 - x1).to_degrees());

        let length = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
        line.set_size(Vector2f::new(length, 1.0));

        self.window.draw(&line);
    }
}

impl <'a> Engine <'a> {
    pub fn update(&self) {}
}

impl <'a> Engine <'a> {
    pub fn new(window: &'a mut RenderWindow, wad: &'a WAD) -> Self {
        let player = Player::new(wad.things[0]);
        let bsp = BSP::new(&wad);

        Self {
            window,
            wad,
            player,
            bsp,
        }
    }
}

// - - -

use sfml::{
    graphics::{
        CircleShape, Color, RectangleShape, RenderTarget, RenderWindow, Shape, Transformable,
    },
    system::Vector2f,
    window::{ContextSettings, Event, Key, Style},
};

pub struct MapViewer <'a> {
    window: &'a mut RenderWindow,

    w_height: f32,
    w_width: f32,

    max_map_height: f32,
    min_map_height: f32,
    max_map_width: f32,
    min_map_width: f32,

    map_vertexes: Vec<Vector2f>,
    map_data:&'a  WAD,

    engine: Engine<'a>,
}

impl <'a> MapViewer <'_> {
    pub fn calc_map_bounds(&mut self) {
        let vertexes = self.map_data.vertexes.to_vec();

        let mut vertexes_x = vertexes.iter().map(|v| v.x).collect::<Vec<_>>();
        let mut vertexes_y = vertexes.iter().map(|v| v.y).collect::<Vec<_>>();

        vertexes_x.sort_by(|a, b| a.partial_cmp(b).unwrap());
        vertexes_y.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let x_size = vertexes_x.len();
        let y_size = vertexes_y.len();

        self.min_map_height = vertexes_y[0] as f32;
        self.max_map_height = vertexes_y[y_size - 1] as f32;

        self.min_map_width = vertexes_x[0] as f32;
        self.max_map_width = vertexes_x[x_size - 1] as f32;
    }
}

impl <'a> MapViewer <'a> {
    pub fn traslate_vertex_x(&self, x: f32) -> f32 {
        (x.min(self.max_map_width).max(self.min_map_width) - self.min_map_width)
            * ((self.w_width - 30.0) - 30.0)
            / (self.max_map_width - self.min_map_width)
            + 30.0
    }

    pub fn traslate_vertex_y(&self, y: f32) -> f32 {
        self.w_height
            - (y.min(self.max_map_height).max(self.min_map_height) - self.min_map_height)
                * ((self.w_height - 30.0) - 30.0)
                / (self.max_map_height - self.min_map_height)
            - 30.0
    }
}

impl <'a> MapViewer <'a> {
    pub fn run(&mut self) {
        let window = &mut self.window;

        let vertexes = &self.map_vertexes;
        let linedefs = &self.map_data.line_defs;

        loop {
            while let Some(event) = window.poll_event() {
                match event {
                    Event::Closed => return,
                    Event::KeyPressed { code, .. } => match code {
                        Key::Escape => return,
                        _ => {}
                    },
                    _ => {}
                }
            }

            window.clear(Color::BLACK);

            // Draw vertexes
            let mut circle = CircleShape::new(2.0, 12);
            circle.set_fill_color(Color::WHITE);

            for vertex in vertexes.iter() {
                let vertex = Vector2f::new(vertex.x - 2.0, vertex.y - 2.0);

                circle.set_position(vertex);
                window.draw(&circle);
            }

            // Draw linedefs
            for linedef in linedefs.iter() {
                let mut line = RectangleShape::new();
                line.set_size(Vector2f::new(1.0, 1.0));
                line.set_fill_color(Color::WHITE);

                let p_1 = vertexes[linedef.start_vertex as usize];
                let p_2 = vertexes[linedef.end_vertex as usize];

                let a = (p_2.y - p_1.y).atan2(p_2.x - p_1.x);

                let length = ((p_2.y - p_1.y).powi(2) + (p_2.x - p_1.x).powi(2)).sqrt();

                line.set_size(Vector2f::new(length, 1.0));
                line.set_rotation(a.to_degrees());
                line.set_position(p_1);

                window.draw(&line);
            }

            // Draw Player
            /*let mut player = CircleShape::new(2.0, 12);

            player.set_fill_color(Color::BLUE);

            let player_x = self.traslate_vertex_x(self.player.position.0);
            let player_y = self.traslate_vertex_y(self.player.position.1);

            player.set_position(Vector2f::new(player_x, player_y));
            player.set_rotation(self.player.angle.to_degrees());

            window.draw(&player);*/
            window.display();
        }
    }
}

impl <'a> MapViewer <'a> {
    pub fn new(width: f32, height: f32, map_data: &'a WAD) -> Self {
        let w_title = "Where's All the Data? - Map Viewer";

        let w_height = height as u32;
        let w_width = width as u32;

        let context = ContextSettings {
            antialiasing_level: 0,
            ..Default::default()
        };

        let mut window = RenderWindow::new((w_width, w_height), w_title, Style::CLOSE, &context);
        window.set_vertical_sync_enabled(true);

        let m_vertexes = map_data.vertexes.clone();

        let player_thing = map_data.things[0];
        
        let player = Player::new(player_thing.clone());
        let bsp = BSP::new(&map_data);

        let engine = Engine::new(&mut window, &map_data);

        let mut viewer = Self {
            window: & mut window,

            w_height: height,
            w_width: width,

            max_map_height: 0.0,
            min_map_height: 0.0,
            max_map_width: 0.0,
            min_map_width: 0.0,

            map_vertexes: Vec::new(),
            map_data,

            engine: engine,
        };

        let mut vertexes = Vec::new();
        viewer.calc_map_bounds();

        for vertex in m_vertexes.iter() {
            let x = viewer.traslate_vertex_x(vertex.x as f32);
            let y = viewer.traslate_vertex_y(vertex.y as f32);

            vertexes.push(Vector2f::new(x, y));
        }

        viewer.map_vertexes = vertexes.clone();
        viewer
    }
}

#[test]
fn test_map_viewer() {
    let mut map_data = WAD::new("/home/flames/Downloads/DOOM.wad").unwrap();
    let _ = map_data.change_map("E1M1");

    let map_viewer = MapViewer::new(320.0 * 4.0, 200.0 * 4.0, &map_data);
    map_viewer.run();
}
