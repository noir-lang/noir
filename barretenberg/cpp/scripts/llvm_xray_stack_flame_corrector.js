// Corrects LLVM-XRAY stack traces to properly line up.
// Otherwise, there is a weird offset in each stack level that does not correspond to any function call.
// In the public domain.
// Conversion of public domain https://github.com/DerickEddington/corrector_of_llvm_xray_stack_flame
class Record {
  constructor(node_path = [], attribute = BigInt(0)) {
    this.node_path = node_path;
    this.attribute = attribute;
  }

  static fromString(line) {
    const components = line.split(";");
    if (components.length >= 2) {
      const attribute = BigInt(components.pop().trim());
      return new Record(components, attribute);
    } else {
      throw new Error("invalid flame format line");
    }
  }

  toString() {
    // To be reversed in-place
    const np = this.node_path.map((x) => x);
    return `${np.reverse().join(";")}; ${this.attribute}`;
  }
}

const Kind = {
  LEAF: "Leaf",
  BRANCH: "Branch",
};

class Node {
  constructor() {
    this.kind = Kind.LEAF;
    this.attribute = null;
    this.children = new Map();
  }

  correctForChild(child) {
    if (this.attribute !== null) {
      this.attribute -= child.attribute;
    }
  }

  child(name) {
    if (this.kind === Kind.LEAF) {
      this.kind = Kind.BRANCH;
      this.children = new Map();
      return this.child(name);
    } else {
      if (!this.children.has(name)) {
        this.children.set(name, new Node());
      }
      return this.children.get(name);
    }
  }

  forEachChild(func) {
    for (let child of this.children.values()) {
      func(this, child);
    }
  }
}

class Tree {
  constructor() {
    this.roots = new Node();
    this.original_order = [];
  }
}

class BadTree extends Tree {
  constructor() {
    super();
  }

  static fromIterator(records) {
    let tree = new BadTree();
    for (let record of records) {
      tree.extend(record);
    }
    return tree;
  }

  extend(record) {
    let parent = this.roots;
    const [lastComponent, ...pathPrefix] = record.node_path.reverse();
    for (let component of pathPrefix.reverse()) {
      parent = parent.child(component);
    }
    const lastNode = parent.child(lastComponent);
    if (lastNode.attribute === null) {
      lastNode.attribute = record.attribute;
      this.original_order.push({ record, node: lastNode });
    } else {
      lastNode.attribute += record.attribute;
    }
  }

  correct() {
    const recur = (parent, child) => {
      parent.correctForChild(child);
      child.forEachChild(recur);
    };
    this.roots.forEachChild((_, root) => root.forEachChild(recur));
    return new GoodTree(this);
  }
}

class GoodTree extends Tree {
  constructor(tree) {
    super();
    this.roots = tree.roots;
    this.original_order = tree.original_order;
  }

  *iter() {
    for (let ordRecord of this.original_order) {
      const { record, node } = ordRecord;
      const originalNodePath = record.node_path;
      const possiblyCorrectedAttribute = node.attribute;
      yield new Record(originalNodePath, possiblyCorrectedAttribute);
    }
  }

  async dump() {
    let output = [];
    for (let record of this.iter()) {
      output.push(`${record.toString()}\n`);
    }
    return output;
  }
}

async function correctStackData(input) {
  const inputRecords = input.map((line) => Record.fromString(line));

  const badTree = BadTree.fromIterator(inputRecords);
  const goodTree = badTree.correct();
  return await goodTree.dump();
}

async function test() {
  const result = await correctStackData([
    "thread1;main; 5925054742",
    "thread1;main;f2; 5925051360",
    "thread1;main;f2;busy; 5925047168",
    "thread1;main; 5941982261",
    "thread1;main;f1; 5941978880",
    "thread1;main;f1;busy; 5941971904",
    "thread1;main; 5930717973",
    "thread1;main;busy; 5930714592",
  ]);
  const expected = [
    "thread1;main; 10144\n",
    "thread1;main;f2; 4192\n",
    "thread1;main;f2;busy; 5925047168\n",
    "thread1;main;f1; 6976\n",
    "thread1;main;f1;busy; 5941971904\n",
    "thread1;main;busy; 5930714592\n",
  ];
  if (JSON.stringify(result) !== JSON.stringify(expected)) {
    throw new Error("test fail");
  }
  console.log("test pass");
}

async function main() {
  // Read standard input
  const inputLines = await new Promise((resolve) => {
    let data = "";
    process.stdin
      .on("data", (chunk) => (data += chunk))
      .on("end", () => resolve(data.split("\n").filter((line) => line)));
  });
  for (const line of await correctStackData(inputLines)) {
    process.stdout.write(line);
  }
}

// test();
main();
