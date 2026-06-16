type NestedMessages = { [key: string]: string | NestedMessages };

export function flattenMessages(obj: NestedMessages, prefix = ""): Record<string, string> {
  const result: Record<string, string> = {};
  for (const [key, value] of Object.entries(obj)) {
    const path = prefix ? `${prefix}.${key}` : key;
    if (typeof value === "string") {
      result[path] = value;
    } else {
      Object.assign(result, flattenMessages(value, path));
    }
  }
  return result;
}

export function unflattenMessages(flat: Record<string, string>): NestedMessages {
  const result: NestedMessages = {};
  for (const [path, value] of Object.entries(flat)) {
    const parts = path.split(".");
    let node = result;
    for (let i = 0; i < parts.length - 1; i++) {
      const part = parts[i];
      if (typeof node[part] !== "object") node[part] = {};
      node = node[part] as NestedMessages;
    }
    node[parts[parts.length - 1]] = value;
  }
  return result;
}
