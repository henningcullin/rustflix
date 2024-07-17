import "./App.css";
import "../app/globals.css";
import { Films } from "./films/Films";
import { DirectoryForm } from "./directories/DirectoryForm";

function App() {
  return (
    <div>
      <DirectoryForm></DirectoryForm>
      <Films></Films>
    </div>
  );
}

export default App;
