import { createRoot } from "react-dom/client";

import { App } from "./App";

import "ft-vox-prototype-0-lib/ft_vox_prototype_0_lib";

const container = document.getElementById("root");
const root = createRoot(container);
const app = <App />;
root.render(app);
