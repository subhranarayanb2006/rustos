# Deploying RustOS to Hugging Face Spaces

You can run **RustOS live in any web browser** on Hugging Face Spaces using **v86** (a pure WebAssembly/JS x86 PC emulator). This allows anyone in the world to boot and interact with RustOS directly without installing VirtualBox!

---

## Ready-to-Deploy Files
All the files needed for Hugging Face are located in the [`huggingface/`](../huggingface/) directory:

1. **`huggingface/README.md`**: Hugging Face Space metadata (`sdk: static`).
2. **`huggingface/index.html`**: Responsive web player interface with keyboard guide, fullscreen mode, and v86 WebAssembly x86 PC engine.
3. **`huggingface/rustos.iso`**: Your compiled bootable ISO (1.2 MB).

---

## Step-by-Step Deployment (Takes 1 Minute)

### Step 1: Create a New Space on Hugging Face
1. Log in to [huggingface.co](https://huggingface.co).
2. Go to [huggingface.co/new-space](https://huggingface.co/new-space).
3. Fill in the details:
   - **Space name**: `rustos` (or any name you like)
   - **License**: `mit`
   - **Select the Space SDK**: Click **Static** (HTML/JS/CSS).
   - **Space hardware**: `Free`
4. Click **Create Space**.

---

### Step 2: Upload the 3 Files
1. Inside your new Space, click the **Files** tab (next to "App").
2. Click **Add file** $\rightarrow$ **Upload files**.
3. Drag and drop the 3 files from your local `huggingface/` folder:
   - `README.md`
   - `index.html`
   - `rustos.iso`
4. Scroll down, type `Initial deploy` under Commit message, and click **Commit changes to main**.

---

### Step 3: Run RustOS in Your Browser!
1. Click the **App** tab at the top of your Space.
2. In 3 seconds, SeaBIOS will initialize, load `rustos.iso`, and your **RustOS Windows-style Desktop** will appear live in the browser!
3. Click anywhere on the screen canvas to capture your keyboard, and use the arrow keys, `[Enter]`, `[S]`, and `[Tab]` to navigate.
