import { NextRequest, NextResponse } from "next/server";

const CERBERUS_API_URL = process.env.CERBERUS_API_URL || "http://localhost:8080/api";

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const status = searchParams.get("status");
    const strategy = searchParams.get("strategy");
    const limit = searchParams.get("limit");

    // Build query parameters
    const params = new URLSearchParams();
    if (status) params.append("status", status);
    if (strategy) params.append("strategy", strategy);
    if (limit) params.append("limit", limit);

    const response = await fetch(
      `${CERBERUS_API_URL}/cerberus/positions?${params.toString()}`,
      {
        headers: {
          "Content-Type": "application/json",
        },
        // Add timeout
        signal: AbortSignal.timeout(10000),
      }
    );

    if (!response.ok) {
      throw new Error(`Cerberus API error: ${response.status} ${response.statusText}`);
    }

    const positions = await response.json();

    return NextResponse.json(positions, {
      headers: {
        "Cache-Control": "no-cache, no-store, must-revalidate",
        "Pragma": "no-cache",
        "Expires": "0",
      },
    });
  } catch (error) {
    console.error("Failed to fetch positions:", error);

    // Return mock data in development
    if (process.env.NODE_ENV === "development") {
      const mockPositions = [
        {
          mint: "So11111111111111111111111111111111111111112",
          symbol: "SOL",
          entryPrice: 23.45,
          currentPrice: 23.67,
          positionSizeSol: 0.1,
          pnl: 0.0094,
          pnlPercent: 0.94,
          status: "open",
          strategy: "sandwich_strategy",
          ageSeconds: 120,
          takeProfitTarget: 100.0,
          stopLossTarget: -25.0,
          timeoutSeconds: 600,
          walletAddress: "DSJXCqXuRckDhSX34oiFgEQChuezxvVgkEAyaA2MML8X",
          createdAt: new Date(Date.now() - 120000).toISOString(),
          updatedAt: new Date().toISOString(),
        },
      ];

      return NextResponse.json(mockPositions);
    }

    return NextResponse.json(
      { error: "Failed to fetch positions", message: error instanceof Error ? error.message : "Unknown error" },
      { status: 500 }
    );
  }
}

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();

    const response = await fetch(`${CERBERUS_API_URL}/cerberus/positions`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(body),
      signal: AbortSignal.timeout(10000),
    });

    if (!response.ok) {
      throw new Error(`Cerberus API error: ${response.status} ${response.statusText}`);
    }

    const result = await response.json();

    return NextResponse.json(result, {
      status: 201,
      headers: {
        "Cache-Control": "no-cache, no-store, must-revalidate",
      },
    });
  } catch (error) {
    console.error("Failed to create position:", error);

    return NextResponse.json(
      { error: "Failed to create position", message: error instanceof Error ? error.message : "Unknown error" },
      { status: 500 }
    );
  }
}
