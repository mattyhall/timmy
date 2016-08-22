use std::iter;

#[derive(Debug, Clone)]
enum CellType {
    Separator,
    Data(String),
}

#[derive(Debug, Clone)]
struct Cell {
    typ: CellType,
    border_left: String,
    border_right: String,
}

impl Cell {
    fn new_left_bordered(t: CellType, border: &str) -> Cell {
        Cell {typ: t, border_left: border.into(), border_right: "".into()}
    }

    fn new_both_bordered(t: CellType, left: &str, right: &str) -> Cell {
        Cell {typ: t, border_left: left.into(), border_right: right.into()}
    }

    fn new_right_bordered(t: CellType, border: &str) -> Cell {
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

struct Table {
    rows: Vec<Vec<Cell>>,
    cols: usize,
}

impl Table {
    fn with_headers(headers: Vec<String>) -> Table {
        let mut table = Table {rows: vec![], cols: headers.len()};
        table.add_border_top();
        table.add_simple(headers);
        table.add_full_separator();
        table
    }

    fn add_row(&mut self, row: Vec<Cell>) {
        self.rows.push(row);
    }

    fn add_simple(&mut self, data: Vec<String>) {
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

    fn add_full_separator(&mut self) {
        self.add_full_separator_custom("├", "┼", "┤");
    }

    fn add_border_top(&mut self) {
        self.add_full_separator_custom("┌", "┬", "┐");
    }

    fn add_border_bottom(&mut self) {
        self.add_full_separator_custom("└", "┴", "┘");
    }

    fn add_full_separator_custom(&mut self, left: &str, middle: &str, right: &str) {
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

    fn print(&self) {
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

fn main() {
    let mut table = Table::with_headers(vec!["Week".into(), "Day  ".into(), "Time".into()]);
    table.add_simple(vec!["22/08/16".into(), "Mon".into(), "1hrs 3mins".into()]);
    table.add_simple(vec!["".into(), "Total".into(), "1hrs 3mins".into()]);
    table.add_full_separator();
    table.add_simple(vec!["15/08/16".into(), "Fri".into(), "55 mins".into()]);
    table.add_simple(vec!["".into(), "Sat".into(), "2hrs 40mins".into()]);
    table.add_simple(vec!["".into(), "Sun".into(), "2hrs 57mins".into()]);
    table.add_simple(vec!["".into(), "Total".into(), "6hrs 33mins".into()]);
    table.add_border_bottom();
    table.print();
}


// ┌──────────┬───────┬─────────────┐
// │ Week     │ Day   │ Time        │
// ├──────────┼───────┼─────────────┤
// │ 22/08/16 │ Mon   │ 1hrs 3mins  │
// │          ┼───────┼─────────────┤
// │          │ Total │ 1hrs 3mins  │
// ├──────────┼───────┼─────────────┤
// │ 15/08/16 │ Fri   │ 55mins      │
// │          │ Sat   │ 2hrs 40mins │
// │          │ Sun   │ 2hrs 57mins │
// ├          │───────┼─────────────┤
// │          │ Total │ 6hrs 33mins │
// └──────────┴───────┴─────────────┘
