import { describe, expect, it } from "vitest";

describe("ambiente de testes", () => {
  it("roda com jsdom", () => {
    expect(typeof document).toBe("object");
  });
});
