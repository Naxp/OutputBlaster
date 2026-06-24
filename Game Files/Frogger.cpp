#include "Frogger.h"

static int WindowsLoop()
{
    // Ticket counter: sdaemon.exe+135C80 (relative offset)
    uint32_t tickets = helpers->ReadInt32(0x135C80, true);

    // Coin P1: sdaemon.exe+C91F8 (relative offset)
    uint32_t coin1 = helpers->ReadInt32(0x0C91F8, true);

    // Coin P2: sdaemon.exe+C9230 (relative offset)
    uint32_t coin2 = helpers->ReadInt32(0x0C9230, true);

    Outputs->SetValue(OutputTicketCounter, (UINT8)tickets);
    Outputs->SetValue(OutputCoin1, (UINT8)coin1);
    Outputs->SetValue(OutputCoin2, (UINT8)coin2);
    Outputs->SetValue(OutputHighScore, (UINT8)tickets);

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
