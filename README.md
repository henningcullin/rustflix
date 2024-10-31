# Rustflix

Rustflix is a desktop application built using React and Tauri that allows users to manage and view their local movie collections. With a modern UI powered by **Shadcn** and **Tailwind CSS**, Rustflix provides an intuitive interface for registering them via IMDb, scraping relevant film information, and having the experience of a service like Netflix in your local folders.

## Features

- **Local Movie Management**: Add a directory of movie files, and Rustflix will sync them to a SQLite database.
- **IMDb Integration**: Easily register your movies by connecting them to IMDb.
- **Automatic Data Scraping**: The application scrapes movie information, including titles, descriptions, and posters, for a complete viewing experience.
- **Responsive Design**: A sleek, responsive UI built with Shadcn and Tailwind CSS.

## Technologies Used

- **Frontend**: React, Shadcn, Tailwind CSS
- **Backend**: Rust, Tauri, Axum
- **Database**: SQLite (bundled rusqlite)

## Installation

### Prerequisites

- Node.js (version 20 or later)
- Rust (latest stable version)
- Tauri CLI

### Steps

1. **Clone the repository:**
```bash
git clone https://github.com/yourusername/rustflix.git
cd rustflix
```

2. **Install dependencies:**
```bash
pnpm i
```

3. **Run the application:**
```bash
pnpm tauri dev
```
