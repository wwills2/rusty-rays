// rendererStorage.ts

export type RendererCache = {
  latestRender?: Uint8Array;
  tracerUuid?: string;
};

const DB_NAME = 'renderer-cache';
const DB_VERSION = 1;
const STORE_NAME = 'cache';

function openDB(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(DB_NAME, DB_VERSION);

    request.onupgradeneeded = () => {
      const db = request.result;
      if (!db.objectStoreNames.contains(STORE_NAME)) {
        db.createObjectStore(STORE_NAME);
      }
    };

    request.onsuccess = () => {
      resolve(request.result);
    };
    request.onerror = () => {
      reject(new Error(request.error?.message || 'Failed to open IndexedDB'));
    };
  });
}

async function saveLatestRender(
  tracerUuid: string,
  data: Uint8Array,
): Promise<void> {
  const blob = new Blob([new Uint8Array(data).slice().buffer], {
    type: 'application/octet-stream',
  });

  const db = await openDB();
  const tx = db.transaction(STORE_NAME, 'readwrite');
  const store = tx.objectStore(STORE_NAME);

  store.put(blob, tracerUuid);

  return new Promise((resolve, reject) => {
    tx.oncomplete = () => {
      resolve();
    };
    tx.onerror = () => {
      reject(new Error(tx.error?.message || 'Failed to save render'));
    };
  });
}

async function loadLatestRender(
  tracerUuid: string,
): Promise<Uint8Array<ArrayBuffer> | undefined> {
  const db = await openDB();
  const tx = db.transaction(STORE_NAME, 'readonly');
  const store = tx.objectStore(STORE_NAME);
  const request = store.get(tracerUuid);

  return new Promise((resolve, reject) => {
    request.onsuccess = async () => {
      const result = request.result as Blob | undefined;
      if (!result) {
        resolve(undefined);
        return;
      }

      try {
        const buffer = await result.arrayBuffer();
        resolve(new Uint8Array(buffer));
      } catch (err) {
        reject(err instanceof Error ? err : new Error('Failed to read Blob'));
      }
    };

    request.onerror = () => {
      reject(new Error(request.error?.message || 'Failed to load render'));
    };
  });
}

async function clearCache(): Promise<void> {
  const db = await openDB();
  const tx = db.transaction(STORE_NAME, 'readwrite');
  tx.objectStore(STORE_NAME).clear();

  return new Promise((resolve, reject) => {
    tx.oncomplete = () => {
      resolve();
    };
    tx.onerror = () => {
      reject(new Error(tx.error?.message || 'Failed to clear cache'));
    };
  });
}

export { saveLatestRender, loadLatestRender, clearCache };
