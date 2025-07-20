import { NextRequest, NextResponse } from "next/server";

const CERBERUS_API_URL = process.env.CERBERUS_API_URL || "http://localhost:8080/api";

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    
    // Validate request body
    if (!body.action || !body.reason) {
      return NextResponse.json(
        { error: "Missing required fields: action and reason" },
        { status: 400 }
      );
    }

    console.log("🚨 PANIC BUTTON TRIGGERED:", {
      action: body.action,
      reason: body.reason,
      timestamp: new Date().toISOString(),
      userAgent: request.headers.get("user-agent"),
      ip: request.headers.get("x-forwarded-for") || request.headers.get("x-real-ip"),
    });

    // Send emergency stop command to Cerberus
    const response = await fetch(`${CERBERUS_API_URL}/cerberus/emergency-stop`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "X-Emergency-Stop": "true",
        "X-Timestamp": new Date().toISOString(),
      },
      body: JSON.stringify({
        action: "EXIT_ALL_POSITIONS",
        reason: body.reason,
        timestamp: body.timestamp || Date.now(),
        source: "frontend_panic_button",
      }),
      // Shorter timeout for emergency operations
      signal: AbortSignal.timeout(5000),
    });

    if (!response.ok) {
      throw new Error(`Cerberus API error: ${response.status} ${response.statusText}`);
    }

    const result = await response.json();

    // Log successful emergency stop
    console.log("✅ EMERGENCY STOP EXECUTED:", {
      success: result.success,
      message: result.message,
      positionsClosed: result.positionsClosed || 0,
      timestamp: new Date().toISOString(),
    });

    return NextResponse.json({
      success: true,
      message: "Emergency stop executed successfully",
      positionsClosed: result.positionsClosed || 0,
      timestamp: new Date().toISOString(),
    }, {
      headers: {
        "Cache-Control": "no-cache, no-store, must-revalidate",
        "X-Emergency-Response": "executed",
      },
    });

  } catch (error) {
    console.error("❌ EMERGENCY STOP FAILED:", error);

    // In development, simulate successful emergency stop
    if (process.env.NODE_ENV === "development") {
      console.log("🔧 Development mode: Simulating emergency stop");
      
      return NextResponse.json({
        success: true,
        message: "Emergency stop executed (development mode)",
        positionsClosed: 3,
        timestamp: new Date().toISOString(),
        development: true,
      });
    }

    return NextResponse.json(
      {
        error: "Emergency stop failed",
        message: error instanceof Error ? error.message : "Unknown error",
        timestamp: new Date().toISOString(),
      },
      { status: 500 }
    );
  }
}
