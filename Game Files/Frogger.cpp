#include "Frogger.h"

static int WindowsLoop()
{
    // --- Known OutputBlaster mappings ---
    uint32_t tickets = helpers->ReadInt32(0x135C80, true);
    uint32_t coin1 = helpers->ReadInt32(0x0C91F8, true);
    uint32_t coin2 = helpers->ReadInt32(0x0C9230, true);

    // --- Bookkeeping / debug variable table (from binary string scan) ---
    uint32_t coinDropsLeft   = helpers->ReadInt32(0x0C9048, true);
    uint32_t coinDropsRight  = helpers->ReadInt32(0x0C9080, true);
    uint32_t serviceCredits  = helpers->ReadInt32(0x0C90B8, true);
    uint32_t ticketsLeft     = helpers->ReadInt32(0x0C90F0, true);
    uint32_t ticketsRight    = helpers->ReadInt32(0x0C9128, true);
    uint32_t creditsLeft     = helpers->ReadInt32(0x0C91D0, true);
    uint32_t creditsRight    = helpers->ReadInt32(0x0C9208, true);
    uint32_t coinsLeft       = helpers->ReadInt32(0x0C9240, true);
    uint32_t coinsRight      = helpers->ReadInt32(0x0C9278, true);

    // --- Game progress variables ---
    uint32_t bugsLeft        = helpers->ReadInt32(0x0C92B0, true);
    uint32_t bugsRight       = helpers->ReadInt32(0x0C92E8, true);
    uint32_t butterfliesLeft  = helpers->ReadInt32(0x0C9320, true);
    uint32_t butterfliesRight = helpers->ReadInt32(0x0C9358, true);

    // --- Lamp / LED reads ---
    uint32_t lamp1 = helpers->ReadInt32(0x41B86BC, true);
    uint32_t lamp3 = helpers->ReadInt32(0x135000, true);
    uint32_t lamp4 = helpers->ReadInt32(0x0C9000, true);

    // --- Send known outputs ---
    Outputs->SetValue(OutputTicketCounter, (UINT8)tickets);
    Outputs->SetValue(OutputCoin1, (UINT8)coin1);
    Outputs->SetValue(OutputCoin2, (UINT8)coin2);
    Outputs->SetValue(OutputHighScore, (UINT8)tickets);

    // --- NEW: Send game progress via existing output slots ---
    Outputs->SetValue(OutputRings, (UINT8)(bugsLeft + bugsRight + butterfliesLeft + butterfliesRight));
    Outputs->SetValue(OutputAmmo1pA, (UINT8)bugsLeft);
    Outputs->SetValue(OutputAmmo1pB, (UINT8)butterfliesLeft);
    Outputs->SetValue(OutputAmmo2pA, (UINT8)bugsRight);
    Outputs->SetValue(OutputAmmo2pB, (UINT8)butterfliesRight);

    // --- NEW: Send bookkeeping via available outputs ---
    Outputs->SetValue(OutputTicketJackpot, (UINT8)ticketsLeft);
    Outputs->SetValue(OutputBase, (UINT8)coinsLeft);
    Outputs->SetValue(OutputExtra, (UINT8)coinsRight);

    // --- Send lamp outputs ---
    Outputs->SetValue(OutputLampStart, lamp1 & 1);
    Outputs->SetValue(OutputLampView1, (lamp1 >> 1) & 1);
    Outputs->SetValue(OutputLampView2, (lamp1 >> 2) & 1);
    Outputs->SetValue(OutputLampView3, lamp3 & 1);
    Outputs->SetValue(OutputLampView4, lamp4 & 1);

    // --- DebugView: full state every cycle ---
    static char msg[512];
    sprintf(msg,
        "OB Frogger: tix=%u c1=%u c2=%u bugsL=%u bugsR=%u bflyL=%u bflyR=%u "
        "credL=%u credR=%u coinL=%u coinR=%u servCred=%u "
        "tixL=%u tixR=%u coinDropL=%u coinDropR=%u "
        "lamps=0x%08X/0x%08X/0x%08X",
        tickets, coin1, coin2,
        bugsLeft, bugsRight, butterfliesLeft, butterfliesRight,
        creditsLeft, creditsRight, coinsLeft, coinsRight, serviceCredits,
        ticketsLeft, ticketsRight, coinDropsLeft, coinDropsRight,
        lamp1, lamp3, lamp4);
    OutputDebugStringA(msg);

    return 0;
}

static DWORD WINAPI OutputsAreGo(LPVOID lpParam)
{
    while (true)
    {
        WindowsLoop();
        Sleep(SleepA);
    }
}

void Frogger::OutputsGameLoop()
{
    if (!init)
    {
        AutoLaunchWinGame();
        Outputs = CreateOutputsFromConfig();
        m_game.name = "Frogger";
        Outputs->SetGame(m_game);
        Outputs->Initialize();
        Outputs->Attached();
        CreateThread(NULL, 0, OutputsAreGo, NULL, 0, NULL);
        while (GetMessage(&Msg1, NULL, NULL, 0))
        {
            TranslateMessage(&Msg1);
            DispatchMessage(&Msg1);
        }
        init = true;
    }
}
