pub mod term {
    use std::io::*;

    use core::time::Duration;
    use crossterm::{
        cursor::{position as crossterm_position, MoveTo, RestorePosition, SavePosition},
        event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers},
        execute, queue,
        style::{Color, Print, SetForegroundColor},
        terminal::{
            disable_raw_mode,
            enable_raw_mode,
            size as crossterm_size,
            //
            Clear,
            ClearType,
        },
    };

    use crate::ui::position::{Position, Visible};
    use crate::ui::screen::Screen;

    macro_rules! ex {
    ( $( $x:expr ),* ) => {
            execute!(
                stdout(),
                $(
                    $x,
                )*
            )
            .unwrap()
        }
    }

    macro_rules! rex {
        ( $( $x:expr ),* ) => {
            execute!(
                stdout(),
                SavePosition,
                $(
                    $x,
                )*
                RestorePosition
            )
            .unwrap();
        };
    }

    const BLANK: char = ' ';

    pub fn free_draw() -> crossterm::Result<()> {
        make_room();
        edit_loop()?;

        Ok(())
    }

    pub fn make_room() {
        ex!(Clear(ClearType::All), MoveTo(0, 0));
    }

    pub fn get_size() -> Position {
        crossterm_size().unwrap_or((0, 0)).into()
    }

    pub fn get_position() -> Position {
        crossterm_position().unwrap_or((0, 0)).into()
    }

    pub fn default_screen() -> Screen {
        Screen::with_size(get_size())
    }

    pub fn event_loop<F>(mut handle_event: F) -> crossterm::Result<()>
    where
        F: FnMut(Position, Res) -> Res,
    {
        enable_raw_mode()?;

        if let Res::Move(dp) = handle_event((0, 0).into(), Res::None) {
            let np: Visible = (dp).into();
            ex!(MoveTo(np.0, np.1))
        }

        loop {
            let size = get_size();
            let cursor = get_position();

            while !poll(Duration::from_millis(500))? {}

            if let Event::Key(event) = read()? {
                let result: Res = key_event_to_res(event, size, cursor);
                let handled = handle_event(cursor, result);

                match handled {
                    Res::Move(dp) => {
                        let np: Visible = (cursor + dp).into();
                        ex!(MoveTo(np.0, np.1))
                    }
                    Res::Quit => break,
                    _ => {}
                }
            }
        }
        disable_raw_mode()?;

        Ok(())
    }

    fn key_event_to_res(
        event: KeyEvent,
        Position { col: w, row: h }: Position,
        Position { col: _c, row: _r }: Position,
    ) -> Res {
        match event {
            // Jump ends
            KeyEvent {
                code: KeyCode::Left,
                modifiers: KeyModifiers::CONTROL,
            } => Res::Move((-w, 0).into()),
            KeyEvent {
                code: KeyCode::Right,
                modifiers: KeyModifiers::CONTROL,
            } => Res::Move((w, 0).into()),

            KeyEvent {
                code: KeyCode::Up,
                modifiers: KeyModifiers::CONTROL,
            } => Res::Move((0, -h).into()),
            KeyEvent {
                code: KeyCode::Down,
                modifiers: KeyModifiers::CONTROL,
            } => Res::Move((0, h).into()),

            // Move
            KeyEvent {
                code: KeyCode::Left,
                modifiers: _,
            } => Res::Move((-1, 0).into()),
            KeyEvent {
                code: KeyCode::Right,
                modifiers: _,
            } => Res::Move((1, 0).into()),

            KeyEvent {
                code: KeyCode::Up,
                modifiers: _,
            } => Res::Move((0, -1).into()),
            KeyEvent {
                code: KeyCode::Down,
                modifiers: _,
            } => Res::Move((0, 1).into()),

            // Quit
            KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
            } => Res::Quit,

            // Enter
            KeyEvent {
                code: KeyCode::Enter,
                modifiers: _,
            } => Res::Enter,

            // Backspace
            KeyEvent {
                code: KeyCode::Backspace,
                modifiers: _,
            } => Res::Backspace,

            // Write
            KeyEvent {
                code: KeyCode::Char(ch),
                modifiers: _,
            } => Res::Write(ch),

            // Unhandled
            _ => Res::None,
        }
    }

    fn edit_loop() -> crossterm::Result<()> {
        let mut screen = Screen::with_size(get_size());
        let mut res: Res = Res::None;

        enable_raw_mode()?;
        loop {
            let size = get_size();
            let cursor = get_position();
            just_dump_screen(&mut screen)?;
            rex!(
                MoveTo(0, 0),
                Clear(ClearType::CurrentLine),
                SetForegroundColor(Color::Red),
                Print(format!(
                    "Size: {} | Pos: {} | Cur: '{}' | Res: {:?}",
                    size,
                    cursor,
                    screen.read(&cursor),
                    res
                ))
            );

            while !poll(Duration::from_millis(500))? {}

            if let Event::Key(event) = read()? {
                rex!(
                    MoveTo(0, 1),
                    Clear(ClearType::CurrentLine),
                    SetForegroundColor(Color::Blue),
                    Print(format!("{:?}", event))
                );

                let result: Res = process_event(&mut screen, event, size, cursor);
                res = result;

                match result {
                    Res::Move(dp) => {
                        let np: Visible = (cursor + dp).into();
                        ex!(MoveTo(np.0, np.1))
                    }
                    Res::Enter => {
                        let np: Visible = Position::new(0, cursor.row + 1).into();
                        ex!(MoveTo(np.0, np.1))
                    }
                    Res::Backspace => {
                        let np: Visible = Position::new(cursor.col - 1, 0).into();
                        ex!(MoveTo(np.0, np.1))
                    }
                    Res::Write(ch) => {
                        screen.write(&cursor, ch);
                    }
                    Res::Quit => break,
                    Res::None => {}
                }
            }
        }
        disable_raw_mode()?;

        Ok(())
    }

    #[derive(Debug, Copy, Clone)]
    pub enum Res {
        Move(Position),
        Write(char),
        Enter,
        Backspace,
        Quit,
        None,
    }

    pub fn process_event(
        screen: &mut Screen,
        event: KeyEvent,
        Position { col: w, row: h }: Position,
        Position { col: c, row: r }: Position,
    ) -> Res {
        match event {
            // Jump ends
            KeyEvent {
                code: KeyCode::Left,
                modifiers: KeyModifiers::CONTROL,
            } => Res::Move((-w, 0).into()),
            KeyEvent {
                code: KeyCode::Right,
                modifiers: KeyModifiers::CONTROL,
            } => Res::Move((w, 0).into()),

            KeyEvent {
                code: KeyCode::Up,
                modifiers: KeyModifiers::CONTROL,
            } => Res::Move((0, -h).into()),
            KeyEvent {
                code: KeyCode::Down,
                modifiers: KeyModifiers::CONTROL,
            } => Res::Move((0, h).into()),

            // Jump boundaries
            KeyEvent {
                code: KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down,
                modifiers: KeyModifiers::ALT,
            } => {
                let mut res = Res::Move((0, 0).into());
                let start_is_blank = screen.read(&(c, r).into()) == BLANK;

                match event.code {
                    KeyCode::Left => {
                        for i in 0..=c {
                            if (screen.read(&(c - i, r).into()) == BLANK) != start_is_blank {
                                res = Res::Move((-i, 0).into());
                                break;
                            }
                        }
                    }
                    KeyCode::Right => {
                        for i in 0..=(w - c) {
                            if (screen.read(&(c + i, r).into()) == BLANK) != start_is_blank {
                                res = Res::Move((i, 0).into());
                                break;
                            }
                        }
                    }
                    KeyCode::Up => {
                        for i in 0..=r {
                            if (screen.read(&(c, r - i).into()) == BLANK) != start_is_blank {
                                res = Res::Move((0, -i).into());
                                break;
                            }
                        }
                    }
                    KeyCode::Down => {
                        for i in 0..=(h - r) {
                            if (screen.read(&(c, r + i).into()) == BLANK) != start_is_blank {
                                res = Res::Move((0, i).into());
                                break;
                            }
                        }
                    }
                    _ => {}
                }
                res
            }

            // Move
            KeyEvent {
                code: KeyCode::Left,
                modifiers: _,
            } => Res::Move((-1, 0).into()),
            KeyEvent {
                code: KeyCode::Right,
                modifiers: _,
            } => Res::Move((1, 0).into()),

            KeyEvent {
                code: KeyCode::Up,
                modifiers: _,
            } => Res::Move((0, -1).into()),
            KeyEvent {
                code: KeyCode::Down,
                modifiers: _,
            } => Res::Move((0, 1).into()),

            // Quit
            KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
            } => Res::Quit,

            // Write
            KeyEvent {
                code: KeyCode::Char(ch),
                modifiers: _,
            } => Res::Write(ch),

            // Unhandled
            _ => Res::None,
        }
    }

    pub fn dump_screen(screen: &mut Screen) -> crossterm::Result<()> {
        enable_raw_mode()?;
        just_dump_screen(screen)?;
        disable_raw_mode()?;

        Ok(())
    }

    pub fn just_dump_screen(screen: &mut Screen) -> crossterm::Result<()> {
        let mut stdout = stdout();
        queue!(stdout, SavePosition)?;
        for (postion, ch) in screen.flush() {
            let clipped: Visible = screen.clamp(&postion).into();
            queue!(stdout, MoveTo(clipped.0, clipped.1), Print(ch))?;
        }
        queue!(stdout, RestorePosition)?;
        stdout.flush()?;

        Ok(())
    }

    #[test]
    fn test_dump() {
        let mut screen = Screen::default();
        screen.write(&(0, 0).into(), 'a');
        screen.write(&(0, 1).into(), 'b');
        screen.write(&(0, 2).into(), 'd');
        screen.write(&(0, 3).into(), 'e');
        screen.write(&(0, 4).into(), 'f');

        screen.write(&(5, 0).into(), 'a');
        screen.write(&(5, 1).into(), 'b');
        screen.write(&(5, 2).into(), 'd');
        screen.write(&(5, 3).into(), 'e');
        screen.write(&(5, 4).into(), 'f');

        screen.write(&(1, 2).into(), 'h');
        screen.write(&(2, 2).into(), 'h');
        screen.write(&(3, 2).into(), 'h');
        screen.write(&(4, 2).into(), 'h');

        make_room();
        dump_screen(&mut screen).unwrap();
        println!();
    }

    pub fn e() {
        enable_raw_mode().unwrap();
    }
    pub fn d() {
        disable_raw_mode().unwrap();
    }

    #[macro_export]
    macro_rules! rawful {
        ($t:expr) => {
            liib::term::e();
            let v = $t;
            liib::term::d();
            v
        };
    }
}

pub mod position {
    use core::cmp::Ordering;
    use core::fmt;
    use core::hash::Hash;
    use std::convert;
    use std::ops;

    pub type Visible = (u16, u16);

    #[derive(Default, Copy, Clone, Debug, Eq)]
    pub struct Position {
        pub col: i32,
        pub row: i32,
    }

    impl Position {
        pub fn new(col: i32, row: i32) -> Self {
            Self { col, row }
        }

        pub fn clamp(&self, min: Position, max: Position) -> Self {
            Self {
                col: self.col.clamp(min.col, max.col),
                row: self.row.clamp(min.row, max.row),
            }
        }
    }

    impl fmt::Display for Position {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "({}, {})", self.col, self.row)
        }
    }

    impl Hash for Position {
        fn hash<H>(&self, state: &mut H)
        where
            H: std::hash::Hasher,
        {
            let t: (i32, i32) = (*self).into();
            t.hash(state);
        }
    }

    impl PartialEq for Position {
        fn eq(&self, other: &Self) -> bool {
            self.row == other.row && self.col == other.col
        }
    }

    impl Ord for Position {
        fn cmp(&self, other: &Self) -> Ordering {
            match self.row.cmp(&other.row) {
                Ordering::Equal => self.col.cmp(&other.col),
                ne_result => ne_result,
            }
        }
    }

    impl PartialOrd for Position {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl ops::Sub for Position {
        type Output = Position;

        fn sub(self, other: Position) -> Position {
            Position {
                col: self.col - other.col,
                row: self.row - other.row,
            }
        }
    }

    impl ops::Add for Position {
        type Output = Position;

        fn add(self, other: Position) -> Position {
            Position {
                col: self.col + other.col,
                row: self.row + other.row,
            }
        }
    }

    impl convert::From<Position> for Visible {
        fn from(p: Position) -> Self {
            let Position { col, row } = p;
            (floorcast(col), floorcast(row))
        }
    }

    fn floorcast(signed: i32) -> u16 {
        if signed < 0 {
            0
        } else {
            signed as u16
        }
    }

    impl convert::From<Visible> for Position {
        fn from(t: Visible) -> Self {
            let (col, row) = t;
            Position {
                // Unsigned bits will always fit, so as casting is fine.
                col: col as i32,
                row: row as i32,
            }
        }
    }

    impl convert::From<Position> for (i32, i32) {
        fn from(p: Position) -> Self {
            let Position { col, row } = p;
            (col, row)
        }
    }

    impl convert::From<(i32, i32)> for Position {
        fn from(t: (i32, i32)) -> Self {
            let (col, row) = t;
            Position { col, row }
        }
    }

    #[test]
    fn test_position() {
        let small = Position::new(1, 2);
        let big = Position::new(3, 6);
        assert!((small + big - small + small) < (big + big));

        let s: u16 = 2 << 14;
        let t: i32 = s as i32;
        let u: u16 = t as u16;
        assert_eq!(s, u);

        let neg = Position::new(-2 << 12, -2 << 6);

        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        fn hash<T: Hash>(t: &T) -> u64 {
            let mut s = DefaultHasher::new();
            t.hash(&mut s);
            s.finish()
        }

        assert!(hash(&neg) == hash(&neg));
        assert!(hash(&neg) != hash(&big));
        assert!(hash(&small) != hash(&big));
        assert!(hash(&small) == hash(&small));
    }
}

pub mod screen {
    use std::collections::HashMap;

    use crate::ui::position::Position;

    pub const BLANK: char = ' ';

    #[derive(Debug)]
    pub struct Screen {
        pub cols: i32,
        pub rows: i32,
        written: HashMap<Position, char>,
        buffer: HashMap<Position, char>,
    }

    impl Default for Screen {
        fn default() -> Self {
            Self::with_size((100, 100).into())
        }
    }

    impl Screen {
        pub fn with_size(col_row: Position) -> Self {
            Self {
                cols: col_row.col,
                rows: col_row.row,
                written: HashMap::with_capacity(10 * 10),
                buffer: HashMap::with_capacity(10 * 10),
            }
        }

        /**
         * Inserts the character into the buffer. Until the screen is [#flush]-ed the written
         * value will not be returned by [#read]. Returns the character that was previously
         * buffered, or [BLANK].
         */
        pub fn write(&mut self, position: &Position, ch: char) -> char {
            self.buffer.insert(*position, ch).unwrap_or(BLANK)
        }

        pub fn clear(&mut self) {
            for &pos in self.written.keys() {
                self.buffer.insert(pos, BLANK);
            }
        }

        /**
         * Returns the character that has been written and flushed at the designated position.
         * If the position has not been touched, [BLANK] is returned.
         */
        pub fn read(&self, position: &Position) -> char {
            match self.written.get(position) {
                Some(&ch) => ch,
                None => BLANK,
            }
        }

        /**
         * Flushes the buffered writes into the written state and returns copies of the elements
         * that were written.
         */
        pub fn flush(&mut self) -> Vec<(Position, char)> {
            let mut updates: Vec<(Position, char)> = Vec::with_capacity(self.buffer.capacity());
            for (&position, &ch) in self.buffer.iter() {
                if self.clamp(&position) != position {
                    // Out-of-bounds positions can be buffered, but the are ignored at flush.
                    continue;
                }

                let original = self.written.insert(position, ch);
                if original == None || original != Some(ch) {
                    updates.push((position, ch));
                }
            }
            updates
        }

        // pub(crate) fn mem(&self) -> usize {
        //     (std::mem::size_of::<Position>() + std::mem::size_of::<char>()) * self.written.len()
        // }

        pub fn clamp(&self, position: &Position) -> Position {
            position.clamp(Position::new(0, 0), Position::new(self.cols, self.rows))
        }
    }

    #[macro_export]
    macro_rules! scrite {
        ($es:expr, $ec: expr, $er: expr, $echars:expr) => {
            {
                let screen: &mut screen::Screen = $es;
                let chars: &str = $echars;

                let mut c: i32 = $ec;
                let mut r: i32 = $er;

                for ch in chars.chars() {
                    if ch == '\n' {
                        r += 1;
                        c = 0;
                    } else {
                        screen.write(&(c, r).into(), ch);
                        c += 1;
                    }
                }
            }
        };
        ($es:expr, $( $x:expr ),+ ) => {
            let screen: &mut Screen = $es;
            $(
                {
                    let (c, r, ch): (i32, i32, char) = $x;
                    screen.write(&(c, r).into(), ch);
                }
            )+
        };
    }

    pub use scrite;

    #[test]
    fn test_buffer() {
        let mut screen = Screen::default();
        scrite!(&mut screen, (0, 1, 'h'), (1, 2, 'i'));

        assert_eq!(screen.read(&(0, 1).into()), ' ');
        assert_eq!(screen.read(&(1, 2).into()), ' ');

        let out = screen.flush();
        assert_eq!(out.len(), 2);

        assert_eq!(screen.read(&(0, 1).into()), 'h');
        assert_eq!(screen.read(&(1, 2).into()), 'i');
    }
}
