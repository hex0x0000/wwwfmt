let nextUserId = 0;

class User {
  #id: number;
  static #nextThreadId: number;

  constructor(id: number) {
    this.#id = id;
  }

  equals(user: this) {
    return this.#id === user.#id;
  }

  async openThread(title: string, content: string): Promise<number> {
    const threadId = User.#nextThreadId;
    await fetch("/createThread", {
      method: "POST",
      body: JSON.stringify({
        content,
        title,
        threadId,
      }),
    });
    return threadId;
  }
}

class Admin extends User {
  #privileges: string[];
  constructor(id: number, privileges: string[] = []) {
    super(id);
    this.#privileges = privileges;
  }

  async closeThread(threadId: number) {
    await fetch("/closeThread", {
      method: "POST",
      body: "" + threadId,
    });
  }
}

const user = new User(nextUserId++);
const admin = new Admin(nextUserId++);

console.log(user.equals(admin));
//@ts-expect-error
console.log(admin.equals(user));

class OptionBuilder<T = string | number | boolean> {
  #options: Map<string, T> = new Map();
  constructor() {}

  add(name: string, value: T): this {
    this.#options.set(name, value);
    return this;
  }

  has(name: string) {
    return this.#options.has(name);
  }

  build() {
    return Object.fromEntries(this.#options);
  }
}

class StringOptionBuilder extends OptionBuilder<string> {
  safeAdd(name: string, value: string) {
    if (!this.has(name)) {
      this.add(name, value);
    }
    return this;
  }
}

const options = new OptionBuilder()
  .add("deflate", true)
  .add("compressionFactor", 10)
  .build();

const languages = new StringOptionBuilder()
  .add("en", "English")
  .safeAdd("de", "Deutsch")
  .safeAdd("de", "German")
  .build();

console.log(languages);
