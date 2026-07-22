import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { LivePerformancePanel } from "./LivePerformancePanel";

describe("LivePerformancePanel", () => {
  it("shows privacy disclaimer", () => {
    render(<LivePerformancePanel />);
    expect(screen.getByText(/Use headphones/i)).toBeInTheDocument();
    expect(screen.getByText(/not an acoustic mouth-to-ear measurement/i)).toBeInTheDocument();
  });

  it("requires headphone acknowledgement before microphone", () => {
    render(<LivePerformancePanel />);
    const micButton = screen.getByRole("button", { name: /Enable Microphone/i });
    expect(micButton).toBeDisabled();
  });

  it("enables microphone after acknowledgement and prepare", () => {
    render(<LivePerformancePanel />);
    fireEvent.click(screen.getByRole("checkbox"));
    fireEvent.click(screen.getByRole("button", { name: /^Prepare$/i }));
    const micButton = screen.getByRole("button", { name: /Enable Microphone/i });
    expect(micButton).not.toBeDisabled();
  });

  it("disables finalized export until completed phase", () => {
    render(<LivePerformancePanel />);
    expect(screen.getByRole("button", { name: /Export Finalized Endpoint/i })).toBeDisabled();
  });
});
