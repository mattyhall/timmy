use std::iter;

#[derive(Debug, Clone)]
pub enum CellType {
    Separator,
    Data(String),
}

#[derive(Debug, Clone)]
pub struct Cell {
    typ: CellType,
    border_left: String,
    border_right: String,
}

impl Cell {
    pub fn new_left_bordered(t: CellType, border: &str) -> Cell {
        Cell {typ: t, border_left: border.into(), border_right: "".into()}
    }

    pub fn new_both_bordered(t: CellType, left: &str, right: &str) -> Cell {
        Cell {typ: t, border_left: left.into(), border_right: right.into()}
    }

    pub fn new_right_bordered(t: CellType, border: &str) -> Cell {
        Cell {typ: t, border_left: "".into(), border_right: border.into()}
    }

    fn print(&self, width: usize) {
        let middle = match self.typ {
            CellType::Separator => iter::repeat("─").take(width+2).collect(),
            CellType::Data(ref s) => {
                let to_pad = width - s.len();
                let spaces: String = iter::repeat(" ").take(to_pad).collect();
                format!(" {}{} ", s, spaces)
            },
        };
        print!("{}{}{}", self.border_left, middle, self.border_right);
    }

    fn len(&self) -> usize {
        match self.typ {
            CellType::Separator => 0,
            CellType::Data(ref s) => s.len(),
        }
    }
}

pub struct Table {
    rows: Vec<Vec<Cell>>,
    cols: usize,
}

impl Table {
    pub fn with_headers(headers: Vec<String>) -> Table {
        let mut table = Table {rows: vec![], cols: headers.len()};
        table.add_border_top();
        table.add_simple(headers);
        table.add_full_separator();
        table
    }

    pub fn add_row(&mut self, row: Vec<Cell>) {
        self.rows.push(row);
    }

    pub fn add_simple(&mut self, data: Vec<String>) {
        let len = data.len();
        let cells =
            data.into_iter()
                .enumerate()
                .map(|(i, data)| {
                    if i != len-1 {
                        Cell::new_left_bordered(CellType::Data(data), "│")
                    } else {
                        Cell::new_both_bordered(CellType::Data(data), "│", "│")
                    }
                })
                .collect();
        self.rows.push(cells);
    }

    pub fn add_full_separator(&mut self) {
        self.add_full_separator_custom("├", "┼", "┤");
    }

    pub fn add_border_top(&mut self) {
        self.add_full_separator_custom("┌", "┬", "┐");
    }

    pub fn add_border_bottom(&mut self) {
        self.add_full_separator_custom("└", "┴", "┘");
    }

    pub fn add_full_separator_custom(&mut self, left: &str, middle: &str, right: &str) {
        let mut cells = vec![];
        let middle_cell = Cell::new_left_bordered(CellType::Separator, middle);
        if self.cols == 1 {
            cells.push(Cell::new_both_bordered(CellType::Separator, left, right));
        } else {
            cells.push(Cell::new_left_bordered(CellType::Separator, left));
        }
        let mut middle_cells: Vec<Cell> = (0..self.cols-2).map(|_| middle_cell.clone()).collect();
        cells.append(&mut middle_cells);
        if self.cols != 1 {
            cells.push(Cell::new_both_bordered(CellType::Separator, middle, right));
        }
        self.rows.push(cells);
    }

    pub fn print(&self) {
        let max_lengths: Vec<usize> = (0..self.cols)
            .map(|i| {
                let lens = self.rows.iter().map(|row| row[i].len());
                lens.max().unwrap()
            })
            .collect();

        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                cell.print(max_lengths[i]);
            }
            println!("");
        }
    }
}

