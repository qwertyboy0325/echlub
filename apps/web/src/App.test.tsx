import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import App from "./App";

vi.mock("./wasm/loader", () => ({
  loadEchlubCore: vi.fn().mockRejectedValue(new Error("WASM module unavailable in test")),
}));

describe("App", () => {
  it("renders foundation disclaimer", () => {
    render(<App />);
    expect(
      screen.getByText("EchLub Laboratory — foundation + performance baseline"),
    ).toBeInTheDocument();
  });

  it("shows wasm unavailable error without typescript domain fallback", async () => {
    render(<App />);
    expect(await screen.findByText(/WASM unavailable/)).toBeInTheDocument();
  });

  it("keeps domain actions disabled when wasm is unavailable", async () => {
    render(<App />);
    await screen.findByText(/WASM unavailable/);
    expect(screen.getByRole("button", { name: "Create track" })).toBeDisabled();
  });
});
