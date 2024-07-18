import "./App.css";
import "../app/globals.css";
import { Films } from "./films/Films";
import { DirectoryTable } from "./directories/DirectoryTable";
import { ThemeProvider } from "./components/ThemeProvider";
import { ModeToggle } from "./components/ModeToggle";

function App() {
  return (
    <ThemeProvider>
      <ModeToggle />
      <DirectoryTable></DirectoryTable>
      <Films></Films>
    </ThemeProvider>
  );
}

export default App;
