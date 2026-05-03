import { Store } from '@tauri-apps/plugin-store';

const STORE_FILE = 'settings.json';

class SettingsState {
  repoPath = $state<string | null>(null);
  activeFirm = $state<string | null>(null);
  loaded = $state(false);

  #store: Store | null = null;

  async load() {
    if (this.loaded) return;
    this.#store = await Store.load(STORE_FILE);
    this.repoPath = (await this.#store.get<string>('repoPath')) ?? null;
    this.activeFirm = (await this.#store.get<string>('activeFirm')) ?? null;
    this.loaded = true;
  }

  async setRepoPath(path: string | null) {
    const changed = path !== this.repoPath;
    this.repoPath = path;
    if (changed) {
      this.activeFirm = null;
      await this.#store?.delete('activeFirm');
    }
    if (path === null) {
      await this.#store?.delete('repoPath');
    } else {
      await this.#store?.set('repoPath', path);
    }
    await this.#store?.save();
  }

  async setActiveFirm(firm: string | null) {
    this.activeFirm = firm;
    if (firm === null) {
      await this.#store?.delete('activeFirm');
    } else {
      await this.#store?.set('activeFirm', firm);
    }
    await this.#store?.save();
  }
}

export const settings = new SettingsState();
