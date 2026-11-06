use crate::user_event::{
    ChannelProfile, CommonChatSummary, ErrorKind, GroupProfile, PhotoResult, UserEvent,
    UserProfile,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Lang {
    #[default]
    En,
    Ru,
}

impl Lang {
    pub fn from_config(s: &str) -> Self {
        let s = s.trim();
        if s.eq_ignore_ascii_case("ru")
            || s.eq_ignore_ascii_case("rus")
            || s.eq_ignore_ascii_case("russian")
            || s == "ру"
            || s.eq_ignore_ascii_case("русский")
        {
            Lang::Ru
        } else {
            Lang::En
        }
    }

    pub fn to_config(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Ru => "ru",
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum I18nKey {
    WindowTitle,
    TabConnect,
    TabLookup,
    TabSettings,
    ApiId,
    ApiHash,
    HintApiPortal,
    TooltipApiPortal,
    TooltipPhone,
    Phone,
    BtnConnect,
    Code,
    BtnSendCode,
    TwoFa,
    BtnSendPassword,
    ConnectSteps,
    Mode,
    Query,
    PhoneConfirm,
    PhoneRisk,
    BtnLookup,
    LookupUsername,
    LookupId,
    LookupPhone,
    LookupUrl,
    OutputDir,
    BtnSave,
    Workdir,
    Language,
    LangEn,
    LangRu,
    SecuritySection,
    SecurityBulletConfig,
    SecurityBulletSession,
    SecurityBulletOutput,
    SecurityBulletUnix,
    SecurityBulletWindows,
    Journal,
    WelcomeLog,
    Busy,
    PreloaderWorking,
    BusyGroupMap,
    WorkerSendFailed,
    FillHashPhone,
    EmptyLookup,
    IdMustBeNumber,
    PhoneCheckboxes,
    SettingsSaved,
    DisclaimerTitle,
    DisclaimerSubtitle,
    DisclaimerAcceptCheckbox,
    DisclaimerBtnAccept,
    DisclaimerBtnDecline,
    PhoneDialogTitle,
    PhoneDialogBody,
    DialogOk,
    DialogCancel,
    ErrorPrefix,
    CardClose,
    CardOpenTme,
    CardNoPhoto,
    CardPhotoError,
    CardSectionDetails,
    CardSectionCommonChats,
    CardCommonChatsNone,
    CardNoPublicLink,
    CardLabelPhone,
    CardLabelStatus,
    CardLabelLanguage,
    CardLabelCalls,
    CardLabelFirstName,
    CardLabelLastName,
    CardLabelUsernames,
    CardLabelBio,
    CardLabelFlags,
    AutoConnectLog,
    CardDeleteHint,
    CardDeleteDialogTitle,
    CardDeleteDialogBody,
    BtnGroupMap,
    GroupMapWindowTitle,
    GroupMapSubtitle,
    GroupMapLegendSelf,
    GroupMapLegendUser,
    GroupMapLegendGroup,
    GroupMapInteractionHint,
    LblGroupMapDepth,
    HintGroupMapDepth,
    GroupMapEmptyDepth,
    ErrGroupMapDepthNaN,
    GroupMapStarted,
}

pub fn window_title() -> String {
    format!("TGScanner {} — full profile", env!("CARGO_PKG_VERSION"))
}

pub fn startup_banner(lang: Lang, workdir: &str) -> String {
    let ver = window_title();
    match lang {
        Lang::En => format!(
            "{ver}\nProgram folder: {workdir}\nIf lookup output has no \"Phone:\" line, you are not running this build.\n\n"
        ),
        Lang::Ru => format!(
            "{ver}\nПапка программы: {workdir}\nЕсли в результате нет строки «Телефон:», запущена не эта сборка.\n\n"
        ),
    }
}

pub fn t(lang: Lang, key: I18nKey) -> &'static str {
    match (lang, key) {
        (Lang::En, I18nKey::WindowTitle) => "TGScanner",
        (Lang::Ru, I18nKey::WindowTitle) => "TGScanner",
        (Lang::En, I18nKey::TabConnect) => "Connection",
        (Lang::Ru, I18nKey::TabConnect) => "Подключение",
        (Lang::En, I18nKey::TabLookup) => "Lookup",
        (Lang::Ru, I18nKey::TabLookup) => "Поиск",
        (Lang::En, I18nKey::TabSettings) => "Settings",
        (Lang::Ru, I18nKey::TabSettings) => "Настройки",
        (Lang::En, I18nKey::ApiId) => "API ID:",
        (Lang::Ru, I18nKey::ApiId) => "API ID:",
        (Lang::En, I18nKey::ApiHash) => "API hash:",
        (Lang::Ru, I18nKey::ApiHash) => "API hash:",
        (Lang::En, I18nKey::HintApiPortal) => "my.telegram.org",
        (Lang::Ru, I18nKey::HintApiPortal) => "my.telegram.org",
        (Lang::En, I18nKey::TooltipApiPortal) => {
            "Open my.telegram.org — sign in, open \"API development tools\", create an app, copy API ID and API hash"
        }
        (Lang::Ru, I18nKey::TooltipApiPortal) => {
            "Открыть my.telegram.org — войти, раздел «API development tools», создать приложение, скопировать API ID и API hash"
        }
        (Lang::En, I18nKey::TooltipPhone) => {
            "Phone linked to your Telegram account (international format, e.g. +79001234567). Login code is sent inside Telegram."
        }
        (Lang::Ru, I18nKey::TooltipPhone) => {
            "Номер, привязанный к аккаунту Telegram (международный формат, напр. +79001234567). Код входа приходит в Telegram."
        }
        (Lang::En, I18nKey::Phone) => "Phone:",
        (Lang::Ru, I18nKey::Phone) => "Телефон:",
        (Lang::En, I18nKey::BtnConnect) => "Connect / request code",
        (Lang::Ru, I18nKey::BtnConnect) => "Подключиться / запросить код",
        (Lang::En, I18nKey::Code) => "Code:",
        (Lang::Ru, I18nKey::Code) => "Код:",
        (Lang::En, I18nKey::BtnSendCode) => "Send code",
        (Lang::Ru, I18nKey::BtnSendCode) => "Отправить код",
        (Lang::En, I18nKey::TwoFa) => "2FA:",
        (Lang::Ru, I18nKey::TwoFa) => "2FA:",
        (Lang::En, I18nKey::BtnSendPassword) => "Send password",
        (Lang::Ru, I18nKey::BtnSendPassword) => "Отправить пароль",
        (Lang::En, I18nKey::ConnectSteps) => {
            "1) Get API ID and hash via the link next to the fields.\n2) Enter phone and connect — code arrives in Telegram.\n3) Enter the code (and 2FA password if prompted)."
        }
        (Lang::Ru, I18nKey::ConnectSteps) => {
            "1) Получите API ID и hash по ссылке у полей.\n2) Укажите телефон и подключитесь — код придёт в Telegram.\n3) Введите код (и пароль 2FA, если потребуется)."
        }
        (Lang::En, I18nKey::Mode) => "Mode:",
        (Lang::Ru, I18nKey::Mode) => "Режим:",
        (Lang::En, I18nKey::Query) => "Query:",
        (Lang::Ru, I18nKey::Query) => "Запрос:",
        (Lang::En, I18nKey::PhoneConfirm) => {
            "I confirm this lookup will temporarily add the number to my Telegram contacts via the API"
        }
        (Lang::Ru, I18nKey::PhoneConfirm) => {
            "Подтверждаю: номер временно будет добавлен в мои контакты Telegram через API"
        }
        (Lang::En, I18nKey::PhoneRisk) => {
            "I confirm lawful purpose, required consent, and compliance with Telegram ToS and data-protection law"
        }
        (Lang::Ru, I18nKey::PhoneRisk) => {
            "Подтверждаю законную цель, необходимые согласия и соблюдение ToS Telegram и законов о персональных данных"
        }
        (Lang::En, I18nKey::BtnLookup) => "Run lookup",
        (Lang::Ru, I18nKey::BtnLookup) => "Выполнить",
        (Lang::En, I18nKey::LookupUsername) => "Username",
        (Lang::Ru, I18nKey::LookupUsername) => "Username",
        (Lang::En, I18nKey::LookupId) => "ID",
        (Lang::Ru, I18nKey::LookupId) => "ID",
        (Lang::En, I18nKey::LookupPhone) => "Phone",
        (Lang::Ru, I18nKey::LookupPhone) => "Телефон",
        (Lang::En, I18nKey::LookupUrl) => "URL",
        (Lang::Ru, I18nKey::LookupUrl) => "URL",
        (Lang::En, I18nKey::OutputDir) => "Output folder:",
        (Lang::Ru, I18nKey::OutputDir) => "Папка output:",
        (Lang::En, I18nKey::BtnSave) => "Save",
        (Lang::Ru, I18nKey::BtnSave) => "Сохранить",
        (Lang::En, I18nKey::Workdir) => "Working folder:",
        (Lang::Ru, I18nKey::Workdir) => "Рабочая папка:",
        (Lang::En, I18nKey::Language) => "Language:",
        (Lang::Ru, I18nKey::Language) => "Язык:",
        (Lang::En, I18nKey::LangEn) => "English",
        (Lang::Ru, I18nKey::LangEn) => "English",
        (Lang::En, I18nKey::LangRu) => "Русский",
        (Lang::Ru, I18nKey::LangRu) => "Русский",
        (Lang::En, I18nKey::SecuritySection) => "Security & local data",
        (Lang::Ru, I18nKey::SecuritySection) => "Безопасность и локальные данные",
        (Lang::En, I18nKey::SecurityBulletConfig) => {
            "• .tguser_config.enc + .tguser_vault.key — encrypted settings (keep both files together)"
        }
        (Lang::Ru, I18nKey::SecurityBulletConfig) => {
            "• .tguser_config.enc + .tguser_vault.key — зашифрованные настройки (храните оба файла вместе)"
        }
        (Lang::En, I18nKey::SecurityBulletSession) => {
            "• .tguser_session.enc — encrypted Telegram session (auth keys)"
        }
        (Lang::Ru, I18nKey::SecurityBulletSession) => {
            "• .tguser_session.enc — зашифрованная сессия Telegram (ключи авторизации)"
        }
        (Lang::En, I18nKey::SecurityBulletOutput) => "• output/ — downloaded photos and JSON",
        (Lang::Ru, I18nKey::SecurityBulletOutput) => "• output/ — загруженные фото и JSON",
        (Lang::En, I18nKey::SecurityBulletUnix) => {
            "• On Unix, vault files are saved with mode 600 when possible"
        }
        (Lang::Ru, I18nKey::SecurityBulletUnix) => {
            "• На Unix файлы vault сохраняются с правами 600, если возможно"
        }
        (Lang::En, I18nKey::SecurityBulletWindows) => {
            "• On Windows, restrict folder access via file properties if needed"
        }
        (Lang::Ru, I18nKey::SecurityBulletWindows) => {
            "• В Windows при необходимости ограничьте доступ к папке в свойствах файлов"
        }
        (Lang::En, I18nKey::Journal) => "Log",
        (Lang::Ru, I18nKey::Journal) => "Журнал",
        (Lang::En, I18nKey::WelcomeLog) => {
            "Welcome. Connect to Telegram on the Connection tab first."
        }
        (Lang::Ru, I18nKey::WelcomeLog) => {
            "Добро пожаловать. Сначала подключитесь к Telegram."
        }
        (Lang::En, I18nKey::AutoConnectLog) => {
            "Auto sign-in: connecting to Telegram…"
        }
        (Lang::Ru, I18nKey::AutoConnectLog) => {
            "Автовход: подключение к Telegram…"
        }
        (Lang::En, I18nKey::Busy) => "Please wait — another operation is in progress…",
        (Lang::Ru, I18nKey::Busy) => "Подождите, выполняется предыдущая операция…",
        (Lang::En, I18nKey::PreloaderWorking) => "Working…",
        (Lang::Ru, I18nKey::PreloaderWorking) => "Выполняется…",
        (Lang::En, I18nKey::BusyGroupMap) => "Building group map (may take a while)…",
        (Lang::Ru, I18nKey::BusyGroupMap) => "Строится карта групп (может занять время)…",
        (Lang::En, I18nKey::WorkerSendFailed) => "Could not send command to worker.",
        (Lang::Ru, I18nKey::WorkerSendFailed) => "Не удалось отправить команду воркеру.",
        (Lang::En, I18nKey::FillHashPhone) => "Enter API hash and phone.",
        (Lang::Ru, I18nKey::FillHashPhone) => "Заполните API hash и телефон.",
        (Lang::En, I18nKey::EmptyLookup) => "Enter a value to look up.",
        (Lang::Ru, I18nKey::EmptyLookup) => "Введите значение для поиска.",
        (Lang::En, I18nKey::IdMustBeNumber) => "ID must be a number.",
        (Lang::Ru, I18nKey::IdMustBeNumber) => "ID должен быть числом.",
        (Lang::En, I18nKey::PhoneCheckboxes) => {
            "For phone lookup, check both confirmations."
        }
        (Lang::Ru, I18nKey::PhoneCheckboxes) => {
            "Для телефона отметьте оба подтверждения."
        }
        (Lang::En, I18nKey::SettingsSaved) => "Settings saved.",
        (Lang::Ru, I18nKey::SettingsSaved) => "Настройки сохранены.",
        (Lang::En, I18nKey::DisclaimerTitle) => "Binding terms, privacy & compliance notice",
        (Lang::Ru, I18nKey::DisclaimerTitle) => "Обязательные условия, конфиденциальность и compliance",
        (Lang::En, I18nKey::DisclaimerSubtitle) => {
            "By continuing you enter a binding agreement with the software terms below"
        }
        (Lang::Ru, I18nKey::DisclaimerSubtitle) => {
            "Продолжая, вы принимаете обязательные условия ниже"
        }
        (Lang::En, I18nKey::DisclaimerAcceptCheckbox) => {
            "I have read and understood the entire notice; I am of legal age and capacity to agree; \
            I am authorized to use the Telegram account and API credentials I provide; I will use TGScanner only lawfully; \
            and I accept these terms as a binding condition of access to the software."
        }
        (Lang::Ru, I18nKey::DisclaimerAcceptCheckbox) => {
            "Я полностью прочитал(а) уведомление; имею возраст и дееспособность для согласия; \
            уполномочен(а) использовать вводимый аккаунт Telegram и API-данные; буду использовать TGScanner только законно; \
            принимаю условия как обязательное основание доступа к программе."
        }
        (Lang::En, I18nKey::DisclaimerBtnAccept) => "Accept and continue",
        (Lang::Ru, I18nKey::DisclaimerBtnAccept) => "Принимаю и продолжаю",
        (Lang::En, I18nKey::DisclaimerBtnDecline) => "Decline and exit",
        (Lang::Ru, I18nKey::DisclaimerBtnDecline) => "Отклонить и выйти",
        (Lang::En, I18nKey::PhoneDialogTitle) => "Confirm phone lookup",
        (Lang::Ru, I18nKey::PhoneDialogTitle) => "Подтверждение поиска по телефону",
        (Lang::En, I18nKey::PhoneDialogBody) => {
            "This action imports the phone number into your Telegram account contacts through the user API, \
            performs the lookup, then attempts to delete the contact. It may create records at Telegram and on this device.\n\n\
            Proceed only if you have a documented lawful basis, any required consent from the data subject, \
            and authority to use this account. You bear full legal responsibility for this operation."
        }
        (Lang::Ru, I18nKey::PhoneDialogBody) => {
            "Действие импортирует номер в контакты вашего аккаунта Telegram через user API, выполняет поиск \
            и пытается удалить контакт. Могут создаваться записи у Telegram и на этом устройстве.\n\n\
            Продолжайте только при документированном правовом основании, необходимых согласиях субъекта данных \
            и полномочиях на использование аккаунта. Вся юридическая ответственность за операцию лежит на вас."
        }
        (Lang::En, I18nKey::DialogOk) => "OK",
        (Lang::Ru, I18nKey::DialogOk) => "OK",
        (Lang::En, I18nKey::DialogCancel) => "Cancel",
        (Lang::Ru, I18nKey::DialogCancel) => "Отмена",
        (Lang::En, I18nKey::ErrorPrefix) => "Error",
        (Lang::Ru, I18nKey::ErrorPrefix) => "Ошибка",
        (Lang::En, I18nKey::CardClose) => "Close",
        (Lang::Ru, I18nKey::CardClose) => "Закрыть",
        (Lang::En, I18nKey::CardOpenTme) => "Open in Telegram",
        (Lang::Ru, I18nKey::CardOpenTme) => "Открыть в Telegram",
        (Lang::En, I18nKey::CardDeleteHint) => "Delete my messages in this chat",
        (Lang::Ru, I18nKey::CardDeleteHint) => "Удалить мои сообщения в этом чате",
        (Lang::En, I18nKey::CardDeleteDialogTitle) => "Delete your messages?",
        (Lang::Ru, I18nKey::CardDeleteDialogTitle) => "Удалить ваши сообщения?",
        (Lang::En, I18nKey::CardDeleteDialogBody) => "TGScanner will scan the full chat history and delete messages you sent. When Telegram allows, they are removed for all participants (not just on this device). This cannot be undone. Large chats may take a long time; FloodWait pauses are normal.",
        (Lang::Ru, I18nKey::CardDeleteDialogBody) => "TGScanner просмотрит всю историю чата и удалит отправленные вами сообщения. Если Telegram позволяет, они исчезнут у всех участников (не только у вас). Действие необратимо. В больших чатах операция может занять много времени; паузы FloodWait — нормальны.",
        (Lang::En, I18nKey::CardNoPhoto) => "No photo",
        (Lang::Ru, I18nKey::CardNoPhoto) => "Нет фото",
        (Lang::En, I18nKey::CardPhotoError) => "Photo unavailable",
        (Lang::Ru, I18nKey::CardPhotoError) => "Фото недоступно",
        (Lang::En, I18nKey::CardSectionDetails) => "Profile",
        (Lang::Ru, I18nKey::CardSectionDetails) => "Профиль",
        (Lang::En, I18nKey::CardSectionCommonChats) => "Common chats",
        (Lang::Ru, I18nKey::CardSectionCommonChats) => "Общие чаты",
        (Lang::En, I18nKey::CardCommonChatsNone) => "No shared chats found",
        (Lang::Ru, I18nKey::CardCommonChatsNone) => "Общих чатов не найдено",
        (Lang::En, I18nKey::CardNoPublicLink) => "(no link)",
        (Lang::Ru, I18nKey::CardNoPublicLink) => "(без ссылки)",
        (Lang::En, I18nKey::CardLabelPhone) => "Phone",
        (Lang::Ru, I18nKey::CardLabelPhone) => "Телефон",
        (Lang::En, I18nKey::CardLabelStatus) => "Status",
        (Lang::Ru, I18nKey::CardLabelStatus) => "Статус",
        (Lang::En, I18nKey::CardLabelLanguage) => "Language",
        (Lang::Ru, I18nKey::CardLabelLanguage) => "Язык",
        (Lang::En, I18nKey::CardLabelCalls) => "Voice / video",
        (Lang::Ru, I18nKey::CardLabelCalls) => "Звонки / видео",
        (Lang::En, I18nKey::CardLabelFirstName) => "First name",
        (Lang::Ru, I18nKey::CardLabelFirstName) => "Имя",
        (Lang::En, I18nKey::CardLabelLastName) => "Last name",
        (Lang::Ru, I18nKey::CardLabelLastName) => "Фамилия",
        (Lang::En, I18nKey::CardLabelUsernames) => "Usernames",
        (Lang::Ru, I18nKey::CardLabelUsernames) => "Usernames",
        (Lang::En, I18nKey::CardLabelBio) => "Bio",
        (Lang::Ru, I18nKey::CardLabelBio) => "Bio",
        (Lang::En, I18nKey::CardLabelFlags) => "Flags",
        (Lang::Ru, I18nKey::CardLabelFlags) => "Метки",
        (Lang::En, I18nKey::BtnGroupMap) => "Group map",
        (Lang::Ru, I18nKey::BtnGroupMap) => "Карта групп",
        (Lang::En, I18nKey::GroupMapWindowTitle) => "Group map — TGScanner",
        (Lang::Ru, I18nKey::GroupMapWindowTitle) => "Карта групп — TGScanner",
        (Lang::En, I18nKey::GroupMapSubtitle) => {
            "Focused graph: enter the target in Lookup (username, numeric ID, or t.me/… URL) — a supergroup/megagroup, or a user (first shared supergroup is used as anchor). Depth 1: anchor participants + which of them share other groups together; depth 2+: also adds more shared groups per participant (API-limited)."
        }
        (Lang::Ru, I18nKey::GroupMapSubtitle) => {
            "Локальный граф: укажите цель в поле «Поиск» (username, числовой ID или t.me/…) — супергруппу/мегагруппу или пользователя (берётся первая общая супергруппа как якорь). Глубина 1: участники якоря и их общие группы между собой; глубина 2+: дополнительно другие общие группы по участникам (ограничения API)."
        }
        (Lang::En, I18nKey::GroupMapLegendSelf) => "● You",
        (Lang::Ru, I18nKey::GroupMapLegendSelf) => "● Вы",
        (Lang::En, I18nKey::GroupMapLegendUser) => "● People",
        (Lang::Ru, I18nKey::GroupMapLegendUser) => "● Люди",
        (Lang::En, I18nKey::GroupMapLegendGroup) => "● Groups",
        (Lang::Ru, I18nKey::GroupMapLegendGroup) => "● Группы",
        (Lang::En, I18nKey::GroupMapInteractionHint) => {
            "Drag with the left mouse button to pan · Mouse wheel to zoom (0.08×–14×)."
        }
        (Lang::Ru, I18nKey::GroupMapInteractionHint) => {
            "Перемещение: зажать левую кнопку мыши и тащить · Масштаб: колёсико (0,08×–14×)."
        }
        (Lang::En, I18nKey::LblGroupMapDepth) => "Map depth (1–5)",
        (Lang::Ru, I18nKey::LblGroupMapDepth) => "Глубина карты (1–5)",
        (Lang::En, I18nKey::HintGroupMapDepth) => {
            "1 = anchor group only; 2+ = also pull more shared supergroups from participants (slower, still capped)."
        }
        (Lang::Ru, I18nKey::HintGroupMapDepth) => {
            "1 = только якорная группа; 2+ = ещё общие супергруппы участников (медленнее, с лимитами)."
        }
        (Lang::En, I18nKey::GroupMapEmptyDepth) => "Enter map depth (number 1–5).",
        (Lang::Ru, I18nKey::GroupMapEmptyDepth) => "Укажите глубину карты (число 1–5).",
        (Lang::En, I18nKey::ErrGroupMapDepthNaN) => "Depth must be a number (1–5).",
        (Lang::Ru, I18nKey::ErrGroupMapDepthNaN) => "Глубина должна быть числом (1–5).",
        (Lang::En, I18nKey::GroupMapStarted) => {
            "Group map: starting (this may take a while; see progress lines below)…"
        }
        (Lang::Ru, I18nKey::GroupMapStarted) => {
            "Карта групп: запуск (может занять время; см. строки прогресса ниже)…"
        }
    }
}

/// One line for the main window journal while building the group map (keeps UI responsive-looking).
pub fn format_group_map_progress_journal(
    lang: Lang,
    message: &str,
    dialogs_scanned: usize,
    groups_done: usize,
    users_seen: usize,
) -> String {
    match lang {
        Lang::En => format!(
            "Group map: {message} (dialogs {dialogs_scanned}, groups {groups_done}, users ~{users_seen})"
        ),
        Lang::Ru => format!(
            "Карта групп: {message} (диалогов {dialogs_scanned}, групп {groups_done}, пользователей ~{users_seen})"
        ),
    }
}

/// Stats line under the group map subtitle.
pub fn format_group_map_stats(
    lang: Lang,
    anchor: &str,
    depth: u32,
    dialogs: usize,
    groups: usize,
    users: usize,
    edges: usize,
    errors: &[String],
) -> String {
    let err_note = if errors.is_empty() {
        String::new()
    } else {
        match lang {
            Lang::En => format!(" Warnings: {}.", errors.len()),
            Lang::Ru => format!(" Предупреждений: {}.", errors.len()),
        }
    };
    match lang {
        Lang::En => format!(
            "Anchor: {anchor}. Depth: {depth}. Dialogs scanned: {dialogs}. Groups processed: {groups}. People: {users}. Links: {edges}.{err_note}"
        ),
        Lang::Ru => format!(
            "Якорь: {anchor}. Глубина: {depth}. Диалогов просмотрено: {dialogs}. Групп обработано: {groups}. Людей: {users}. Связей: {edges}.{err_note}"
        ),
    }
}

/// Full startup disclaimer text (scrollable body).
pub fn disclaimer_document(lang: Lang) -> String {
    match lang {
        Lang::En => concat!(
            "IMPORTANT: This notice is a binding agreement. It does not constitute legal advice. \
            Obtain independent counsel if you are unsure of your obligations.\n\n",
            "1. Software status\n",
            "TGScanner is independent open-source software published by WtekSupport under the GNU Affero General Public License v3.0 or later. \
            Telegram, the Telegram logo, and related marks are trademarks of their respective owners; TGScanner is not affiliated with, \
            sponsored by, or endorsed by Telegram FZ-LLC or its affiliates. The software is a general-purpose client tool; \
            authors and contributors do not operate the service, do not receive your queries, and do not control your Telegram account.\n\n",
            "2. Eligibility and authority\n",
            "You represent and warrant that: (a) you are of legal age and capacity to enter binding agreements in your jurisdiction; \
            (b) you are the lawful user of each Telegram account and each API ID/hash you enter, or you have explicit written authorization from the account/API owner; \
            (c) your use will not violate export-control, sanctions, or anti-corruption laws applicable to you.\n\n",
            "3. Acceptable use\n",
            "You may use TGScanner only for lawful purposes and with a valid legal basis under applicable law. \
            Without limitation, you must NOT use it to: harass, threaten, or stalk any person; conduct unauthorized surveillance or employee monitoring without proper notices and bases; \
            bypass technical limits or rate limits; scrape or bulk-collect data in breach of Telegram rules; \
            identify or profile individuals for discriminatory, fraudulent, or unlawful purposes; or process special categories of personal data unless explicitly permitted by law with all required safeguards.\n\n",
            "4. Telegram platform rules\n",
            "Your use of Telegram through this software remains subject to Telegram Terms of Service (https://telegram.org/tos), \
            Telegram Privacy Policy (https://telegram.org/privacy), and the rules published at https://my.telegram.org. \
            You are solely responsible for compliance. Telegram may suspend or terminate accounts or API access at its discretion. \
            WtekSupport and contributors are not responsible for Telegram’s actions or service availability.\n\n",
            "5. Personal data — your role as controller\n",
            "When you run lookups, you (not the software authors) determine the purposes and means of processing personal data you obtain. \
            You act as an independent data controller (or as an authorized processor acting on documented instructions). \
            Depending on your location, you must comply with applicable data-protection laws (including, where relevant, the EU/UK GDPR, \
            U.S. state privacy laws, and similar regimes): lawful basis, transparency, data minimization, purpose limitation, accuracy, \
            storage limitation, integrity and confidentiality, records of processing, data subject rights, cross-border transfer rules, \
            and breach notification duties. Where consent is your basis, it must be valid, informed, specific, and documented. \
            TGScanner does not provide a legal basis for you; you must establish one before processing.\n\n",
            "6. Local data on this device\n",
            "The application stores locally, in the program folder: encrypted configuration (.tguser_config.enc), encrypted session material (.tguser_session.enc), \
            a local encryption key (.tguser_vault.key), auxiliary state files, and exports under output/ (e.g. photos, JSON). \
            These may contain authentication secrets and personal data relating to third parties. Encryption reduces casual access but is not a substitute for your security practices. \
            You must restrict filesystem access, use full-disk encryption where appropriate, back up responsibly, and securely erase data when retention is no longer justified. \
            Do not distribute, publish, or transfer the application folder or outputs except as lawfully permitted.\n\n",
            "7. Phone number lookup (heightened risk)\n",
            "Phone lookup uses Telegram’s contact-import mechanism: the number is added to your account contacts, processed, and the software attempts removal afterward. \
            This may affect the data subject’s experience on Telegram and creates processing records. Use only where strictly necessary, with documented lawful basis and any required consent. \
            The Lookup tab requires additional confirmations; those confirmations do not replace your legal obligations.\n\n",
            "8. Intellectual property\n",
            "Third-party materials (Telegram, grammers, Slint, etc.) remain subject to their licenses. You receive no ownership rights in TGScanner except as granted by AGPL-3.0-or-later. \
            If you modify or offer network access to modified versions, you must comply with AGPL obligations.\n\n",
            "9. Disclaimer of warranties\n",
            "TO THE MAXIMUM EXTENT PERMITTED BY APPLICABLE LAW, TGSCANNER IS PROVIDED \"AS IS\" AND \"AS AVAILABLE\" WITHOUT WARRANTIES OF ANY KIND, \
            WHETHER EXPRESS, IMPLIED, OR STATUTORY, INCLUDING IMPLIED WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, \
            NON-INFRINGEMENT, ACCURACY, OR THAT USE WILL BE UNINTERRUPTED, ERROR-FREE, OR COMPLIANT WITH ANY LAW.\n\n",
            "10. Limitation of liability\n",
            "TO THE MAXIMUM EXTENT PERMITTED BY APPLICABLE LAW, IN NO EVENT SHALL WTEKSUPPORT, TGSCANNER AUTHORS, OR CONTRIBUTORS BE LIABLE FOR ANY INDIRECT, \
            INCIDENTAL, SPECIAL, CONSEQUENTIAL, EXEMPLARY, OR PUNITIVE DAMAGES, OR ANY LOSS OF PROFITS, DATA, GOODWILL, OR BUSINESS, \
            ARISING FROM OR RELATED TO YOUR USE OR INABILITY TO USE THE SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY. \
            WHERE LIABILITY CANNOT BE EXCLUDED, THE AGGREGATE LIABILITY SHALL NOT EXCEED THE GREATER OF (A) AMOUNTS YOU PAID FOR THE SOFTWARE (IF ANY) IN THE TWELVE MONTHS BEFORE THE CLAIM OR (B) USD 100.\n\n",
            "11. Indemnification\n",
            "You agree to indemnify, defend, and hold harmless WtekSupport and TGScanner contributors from and against any claims, investigations, fines, penalties, damages, losses, \
            and expenses (including reasonable attorneys’ fees) arising out of or related to: your use or misuse of TGScanner; personal data you process; \
            breach of this notice; violation of Telegram terms; or violation of applicable law. We may assume exclusive defense of any matter subject to indemnification; you will cooperate.\n\n",
            "12. Changes; severability; governing law\n",
            "This notice may be updated in future releases. Where permitted by law, continued use after an update constitutes acceptance of the revised notice. \
            If any provision is held invalid, the remainder stays in effect. This notice is governed by the laws applicable in your jurisdiction to the extent not overridden by mandatory consumer protections; \
            disputes are your responsibility to resolve with affected parties and authorities — the software is provided without undertaking to represent you.\n\n",
            "13. Acknowledgment\n",
            "If you do not agree to every section above, you must decline and exit. Acceptance via the checkbox and \"Accept and continue\" confirms your agreement."
        )
        .to_string(),
        Lang::Ru => concat!(
            "ВАЖНО: данный текст — обязательное соглашение. Он не заменяет консультацию юриста. \
            При сомнениях обратитесь к независимому специалисту.\n\n",
            "1. Статус программы\n",
            "TGScanner — независимое ПО с открытым исходным кодом (WtekSupport), лицензия GNU Affero GPL v3.0 или новее. \
            Telegram и связанные обозначения принадлежат правообладателям; TGScanner не аффилирован с Telegram FZ-LLC и не одобрен им. \
            Авторы не оказывают услуги Telegram, не получают ваши запросы и не управляют вашим аккаунтом.\n\n",
            "2. Дееспособность и полномочия\n",
            "Вы заявляете и гарантируете, что: (a) достигли возраста и обладаете дееспособностью для заключения соглашений по законам вашей юрисдикции; \
            (b) являетесь законным пользователем каждого аккаунта Telegram и каждой пары API ID/hash либо имеете письменное разрешение владельца; \
            (c) использование не нарушит экспортный контроль, санкции и антикоррупционные нормы, применимые к вам.\n\n",
            "3. Допустимое использование\n",
            "Используйте TGScanner только законно и при наличии правового основания. Запрещено, в частности: преследование, угрозы, stalking; \
            несанкционированное наблюдение или мониторинг без уведомлений и оснований; обход лимитов; массовый сбор данных с нарушением правил Telegram; \
            профилирование в дискриминационных, мошеннических или незаконных целях; обработка специальных категорий персональных данных без явного разрешения закона и мер защиты.\n\n",
            "4. Правила платформы Telegram\n",
            "Использование Telegram через программу регулируется ToS (https://telegram.org/tos), политикой конфиденциальности (https://telegram.org/privacy) \
            и правилами https://my.telegram.org. Соблюдение — ваша исключительная обязанность. Telegram может ограничить или прекратить доступ. \
            WtekSupport и участники проекта не отвечают за действия Telegram и доступность сервиса.\n\n",
            "5. Персональные данные — ваша роль оператора\n",
            "При поиске вы (а не авторы ПО) определяете цели и средства обработки получаемых персональных данных. \
            Вы выступаете самостоятельным оператором (или уполномоченным обработчиком по документированным инструкциям). \
            Соблюдайте применимое законодательство (в т.ч. GDPR/Великобритания, законы штатов США о privacy, ФЗ-152 и иные акты): \
            законность, прозрачность, минимизация, ограничение цели, точность, сроки хранения, безопасность, учёт операций, права субъектов, трансграничная передача, уведомление об инцидентах. \
            Согласие, если им пользуетесь, должно быть действительным, информированным, конкретным и задокументированным. \
            Программа не создаёт для вас правовое основание — вы обязаны установить его до обработки.\n\n",
            "6. Локальные данные на устройстве\n",
            "В папке программы хранятся: зашифрованная конфигурация (.tguser_config.enc), сессия (.tguser_session.enc), ключ (.tguser_vault.key), \
            служебные файлы и экспорт в output/ (фото, JSON). В них могут быть секреты и персональные данные третьих лиц. \
            Шифрование не заменяет ваши меры безопасности. Ограничивайте доступ к ФС, используйте шифрование диска, резервное копирование и безопасное удаление по истечении сроков. \
            Не распространяйте папку приложения и выгрузки без законного основания.\n\n",
            "7. Поиск по номеру телефона (повышенный риск)\n",
            "Используется импорт контакта в аккаунт Telegram, затем попытка удаления. Это может затрагивать субъекта данных и создавать записи обработки. \
            Применяйте только при необходимости, с документированным основанием и согласиями. Подтверждения на вкладке «Поиск» не снимают юридических обязанностей.\n\n",
            "8. Интеллектуальная собственность\n",
            "Компоненты третьих лиц (Telegram, grammers, Slint и др.) подчиняются своим лицензиям. Права на TGScanner — по AGPL-3.0-or-later. \
            При модификации и сетевом предоставлении изменённых версий соблюдайте обязательства AGPL.\n\n",
            "9. Отказ от гарантий\n",
            "В МАКСИМАЛЬНО ДОПУСТИМОЙ ЗАКОНОМ СТЕПЕНИ ПО ПРЕДОСТАВЛЯЕТСЯ «КАК ЕСТЬ» БЕЗ КАКИХ-ЛИБО ГАРАНТИЙ, ЯВНЫХ ИЛИ ПОДРАЗУМЕВАЕМЫХ, \
            ВКЛЮЧАЯ ТОВАРНУЮ ПРИГОДНОСТЬ, ПРИГОДНОСТЬ ДЛЯ ЦЕЛИ, НЕНАРУШЕНИЕ ПРАВ, ТОЧНОСТЬ, БЕСПЕРЕБОЙНОСТЬ И СООТВЕТСТВИЕ ЗАКОНУ.\n\n",
            "10. Ограничение ответственности\n",
            "В МАКСИМАЛЬНО ДОПУСТИМОЙ СТЕПЕНИ WTEKSUPPORT, АВТОРЫ И УЧАСТНИКИ НЕ НЕСУТ ОТВЕТСТВЕННОСТИ ЗА КОСВЕННЫЙ, СЛУЧАЙНЫЙ, СПЕЦИАЛЬНЫЙ, \
            ПОСЛЕДУЮЩИЙ УЩЕРБ, УПУЩЕННУЮ ВЫГОДУ, ПОТЕРЮ ДАННЫХ ИЛИ РЕПУТАЦИИ, ДАЖЕ ПРИ ПРЕДУПРЕЖДЕНИИ О РИСКЕ. \
            ЕСЛИ ОТВЕТСТВЕННОСТЬ НЕ МОЖЕТ БЫТЬ ИСКЛЮЧЕНА, СОВОКУПНЫЙ РАЗМЕР — НЕ БОЛЕЕ БОЛЬШЕГО ИЗ: (A) СУММЫ, УПЛАЧЕННОЙ ВАМИ ЗА ПО (ЕСЛИ БЫЛА) ЗА 12 МЕСЯЦЕВ ДО ПРЕТЕНЗИИ, (B) 100 USD.\n\n",
            "11. Возмещение убытков (indemnity)\n",
            "Вы обязуетесь возместить и оградить WtekSupport и участников проекта от претензий, проверок, штрафов, убытков и расходов (включая разумные гонорары юристов), \
            связанных с вашим использованием или злоупотреблением TGScanner, обработкой персональных данных, нарушением настоящего текста, условий Telegram или закона.\n\n",
            "12. Изменения; делимость; применимое право\n",
            "Текст может обновляться в новых версиях. Где законом допускается, продолжение использования означает принятие новой редакции. \
            Недействительность отдельного положения не отменяет остальные. Применяются императивные нормы вашей юрисдикции; споры с третьими лицами и органами — ваша зона ответственности.\n\n",
            "13. Подтверждение\n",
            "Если вы не согласны со всеми разделами — отклоните и выйдите. Отметка чекбокса и «Принимаю и продолжаю» означает полное согласие."
        )
        .to_string(),
    }
}

pub fn format_event(lang: Lang, ev: &UserEvent) -> String {
    match ev {
        UserEvent::Ready { account } => match lang {
            Lang::En => format!("Signed in as {account}. Lookups are enabled."),
            Lang::Ru => format!("Вход подтверждён: {account}. Поиск доступен."),
        },
        UserEvent::SessionLost => match lang {
            Lang::En => {
                "Connection lost or session invalid. Connect again on the Connection tab.".into()
            }
            Lang::Ru => {
                "Соединение разорвано или сессия недействительна. Подключитесь снова на вкладке «Подключение».".into()
            }
        },
        UserEvent::CodeSent => match lang {
            Lang::En => {
                "Code sent in Telegram. Enter it on the Connection tab.".into()
            }
            Lang::Ru => {
                "Код отправлен в Telegram. Введите его на вкладке «Подключение».".into()
            }
        },
        UserEvent::Need2fa => match lang {
            Lang::En => "2FA password required.".into(),
            Lang::Ru => "Нужен пароль 2FA.".into(),
        },
        UserEvent::UserProfile(p) => format_user_profile(lang, p),
        UserEvent::GroupProfile(g) => format_group_profile(lang, g),
        UserEvent::ChannelProfile(c) => format_channel_profile(lang, c),
        UserEvent::MessageExport {
            msg_id,
            text,
            json_path,
        } => match lang {
            Lang::En => format!(
                "Message #{msg_id}\n{text}\nJSON: {}",
                json_path.display()
            ),
            Lang::Ru => format!(
                "Сообщение #{msg_id}\n{text}\nJSON: {}",
                json_path.display()
            ),
        },
        UserEvent::PhoneNotFound { phone } => match lang {
            Lang::En => format!("No user found for phone {phone}."),
            Lang::Ru => format!("Пользователь с номером {phone} не найден."),
        },
        UserEvent::PhoneLookupComplete {
            profile,
            contact_removed,
        } => {
            let mut s = format_event(lang, profile);
            if *contact_removed {
                s.push('\n');
                s.push_str(match lang {
                    Lang::En => "Temporary contact removed (if the API allowed).",
                    Lang::Ru => "Временный контакт удалён (если API позволило).",
                });
            }
            s
        }
        UserEvent::DeleteMessagesProgress { .. } => String::new(),
        UserEvent::GroupMapProgress {
            message,
            dialogs_scanned,
            groups_done,
            users_seen,
        } => match lang {
            Lang::En => format!(
                "Group map: {message} (dialogs {dialogs_scanned}, groups {groups_done}, users ~{users_seen})"
            ),
            Lang::Ru => format!(
                "Карта групп: {message} (диалогов {dialogs_scanned}, групп {groups_done}, пользователей ~{users_seen})"
            ),
        },
        UserEvent::GroupMapReady(m) => match lang {
            Lang::En => format!(
                "Group map ready — anchor: {}, depth {} — {} dialogs, {} groups, {} people ({} edges).",
                m.anchor_label,
                m.depth,
                m.dialogs_scanned,
                m.groups_processed,
                m.users_found,
                m.edges.len()
            ),
            Lang::Ru => format!(
                "Карта групп готова — якорь: {}, глубина {} — {} диалогов, {} групп, {} людей (связей {}).",
                m.anchor_label,
                m.depth,
                m.dialogs_scanned,
                m.groups_processed,
                m.users_found,
                m.edges.len()
            ),
        },
        UserEvent::DeleteMessagesFinished {
            chat_title,
            scanned,
            deleted,
            failed_batches,
        } => format_delete_finished(lang, chat_title, *scanned, *deleted, *failed_batches),
        UserEvent::Error { kind, detail } => format_error(lang, *kind, detail),
    }
}

fn format_error(lang: Lang, kind: ErrorKind, detail: &str) -> String {
    let msg = match (lang, kind) {
        (Lang::En, ErrorKind::NotConnected) => {
            "Connect to Telegram first".to_string()
        }
        (Lang::Ru, ErrorKind::NotConnected) => "Сначала подключитесь к Telegram".into(),
        (Lang::En, ErrorKind::RequestCodeFirst) => "Request a login code first".into(),
        (Lang::Ru, ErrorKind::RequestCodeFirst) => "Сначала запросите код".into(),
        (Lang::En, ErrorKind::ClientNotConnected) => "Client not connected".into(),
        (Lang::Ru, ErrorKind::ClientNotConnected) => "Клиент не подключён".into(),
        (Lang::En, ErrorKind::NoPasswordRequest) => "No password prompt pending".into(),
        (Lang::Ru, ErrorKind::NoPasswordRequest) => "Нет запроса пароля".into(),
        (Lang::En, ErrorKind::AccountNotRegistered) => "Account not registered".into(),
        (Lang::Ru, ErrorKind::AccountNotRegistered) => "Аккаунт не зарегистрирован".into(),
        (Lang::En, ErrorKind::UnsupportedId) => "Unsupported ID format".into(),
        (Lang::Ru, ErrorKind::UnsupportedId) => "Неподдерживаемый формат ID".into(),
        (Lang::En, ErrorKind::ChannelNotFound) => "Channel not found".into(),
        (Lang::Ru, ErrorKind::ChannelNotFound) => "Канал не найден".into(),
        (Lang::En, ErrorKind::MessageUnavailable) => "Message not available".into(),
        (Lang::Ru, ErrorKind::MessageUnavailable) => "Сообщение недоступно".into(),
        (Lang::En, ErrorKind::InvalidUrl) => {
            "URL must look like https://t.me/channel/123".into()
        }
        (Lang::Ru, ErrorKind::InvalidUrl) => {
            "URL должен быть вида https://t.me/канал/123".into()
        }
        (Lang::En, ErrorKind::InvalidUrlPath) => "Invalid path in URL".into(),
        (Lang::Ru, ErrorKind::InvalidUrlPath) => "Неверный путь в URL".into(),
        (Lang::En, ErrorKind::InvalidChannelName) => "Invalid channel name".into(),
        (Lang::Ru, ErrorKind::InvalidChannelName) => "Недопустимое имя канала".into(),
        (Lang::En, ErrorKind::InvalidMessageId) => "Message ID must be > 0".into(),
        (Lang::Ru, ErrorKind::InvalidMessageId) => "ID сообщения должен быть > 0".into(),
        (Lang::En, ErrorKind::InvalidPhoneE164) => {
            "Phone must be E.164, e.g. +79001234567".into()
        }
        (Lang::Ru, ErrorKind::InvalidPhoneE164) => {
            "Номер в формате E.164, например +79001234567".into()
        }
        (Lang::En, ErrorKind::NotFoundUsername) if detail.is_empty() => {
            "User not found".into()
        }
        (Lang::Ru, ErrorKind::NotFoundUsername) if detail.is_empty() => "Не найдено".into(),
        (Lang::En, ErrorKind::NotFoundUsername) => format!("Not found: {detail}"),
        (Lang::Ru, ErrorKind::NotFoundUsername) => format!("Не найдено: {detail}"),
        (_, ErrorKind::UnpackChat) if !detail.is_empty() => detail.to_string(),
        (Lang::En, ErrorKind::UnpackChat) => {
            "Could not resolve ID (users may need access hash via username/contacts).".into()
        }
        (Lang::Ru, ErrorKind::UnpackChat) => {
            "Не удалось разрешить ID (для пользователей нужен access hash — контакты/username).".into()
        }
        (_, k) if matches!(k, ErrorKind::Session | ErrorKind::Connect | ErrorKind::RequestCode | ErrorKind::SignIn | ErrorKind::CheckPassword | ErrorKind::SaveSession | ErrorKind::ImportContacts | ErrorKind::Generic) && !detail.is_empty() => {
            detail.to_string()
        }
        (_, ErrorKind::Session) => "Session error".into(),
        (_, ErrorKind::Connect) => "Connection error".into(),
        (_, ErrorKind::RequestCode) => "Could not request code".into(),
        (_, ErrorKind::SignIn) => "Sign-in error".into(),
        (_, ErrorKind::CheckPassword) => "Password check failed".into(),
        (_, ErrorKind::SaveSession) => "Could not save session".into(),
        (_, ErrorKind::ImportContacts) => "Contact import failed".into(),
        (_, ErrorKind::Generic) => detail.to_string(),
    };
    if matches!(
        kind,
        ErrorKind::Session
            | ErrorKind::Connect
            | ErrorKind::RequestCode
            | ErrorKind::SignIn
            | ErrorKind::CheckPassword
            | ErrorKind::SaveSession
            | ErrorKind::ImportContacts
            | ErrorKind::Generic
    ) && !detail.is_empty()
        && !msg.contains(detail)
    {
        format!("{}: {msg}", t(lang, I18nKey::ErrorPrefix))
    } else if detail.is_empty() || msg == detail {
        format!("{}: {msg}", t(lang, I18nKey::ErrorPrefix))
    } else if msg.starts_with("session:")
        || msg.starts_with("connect:")
        || msg.contains("sign_in:")
        || msg.contains("check_password:")
        || msg.contains("import:")
    {
        format!("{}: {msg}", t(lang, I18nKey::ErrorPrefix))
    } else {
        format!("{}: {msg}", t(lang, I18nKey::ErrorPrefix))
    }
}

fn dash(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "—",
        Lang::Ru => "—",
    }
}

fn format_photo(lang: Lang, photo: &PhotoResult) -> String {
    match photo {
        PhotoResult::None => match lang {
            Lang::En => "Profile photo: none".into(),
            Lang::Ru => "Фото профиля: нет".into(),
        },
        PhotoResult::Saved(p) => match lang {
            Lang::En => format!("Photo: {}", p.display()),
            Lang::Ru => format!("Фото: {}", p.display()),
        },
        PhotoResult::Failed(e) => match lang {
            Lang::En => format!("(photo: {e})"),
            Lang::Ru => format!("(фото: {e})"),
        },
    }
}

fn fmt_opt(lang: Lang, value: &Option<String>) -> String {
    value
        .as_ref()
        .filter(|s| !s.is_empty())
        .cloned()
        .unwrap_or_else(|| dash(lang).into())
}

fn fmt_opt_i32(lang: Lang, value: Option<i32>) -> String {
    value
        .map(|n| n.to_string())
        .unwrap_or_else(|| dash(lang).into())
}

fn fmt_opt_bool(lang: Lang, value: Option<bool>) -> String {
    match value {
        Some(true) => match lang {
            Lang::En => "yes".into(),
            Lang::Ru => "да".into(),
        },
        Some(false) => match lang {
            Lang::En => "no".into(),
            Lang::Ru => "нет".into(),
        },
        None => dash(lang).into(),
    }
}

fn fmt_list(lang: Lang, items: &[String]) -> String {
    if items.is_empty() {
        dash(lang).into()
    } else {
        items.join(", ")
    }
}

fn fmt_bool(lang: Lang, value: bool) -> &'static str {
    match (lang, value) {
        (Lang::En, true) => "yes",
        (Lang::En, false) => "no",
        (Lang::Ru, true) => "да",
        (Lang::Ru, false) => "нет",
    }
}

pub fn display_opt(lang: Lang, value: &Option<String>) -> String {
    fmt_opt(lang, value)
}

pub fn format_usernames(p: &UserProfile) -> String {
    if !p.usernames.is_empty() {
        p.usernames
            .iter()
            .map(|u| format!("@{u}"))
            .collect::<Vec<_>>()
            .join(", ")
    } else if let Some(u) = &p.username {
        format!("@{u}")
    } else {
        "—".to_string()
    }
}

pub fn format_calls_pair(lang: Lang, p: &UserProfile) -> String {
    format!(
        "{} / {}",
        fmt_opt_bool(lang, p.phone_calls_available),
        fmt_opt_bool(lang, p.video_calls_available)
    )
}

pub fn format_profile_flags(lang: Lang, p: &UserProfile) -> String {
    let mut parts = Vec::new();
    if p.is_bot {
        parts.push(match lang {
            Lang::En => "bot",
            Lang::Ru => "бот",
        });
    }
    if p.scam {
        parts.push("scam");
    }
    if p.fake {
        parts.push("fake");
    }
    if p.restricted {
        parts.push(match lang {
            Lang::En => "restricted",
            Lang::Ru => "ограничен",
        });
    }
    if p.deleted {
        parts.push(match lang {
            Lang::En => "deleted",
            Lang::Ru => "удалён",
        });
    }
    parts.join(" · ")
}

pub fn format_delete_progress_title(lang: Lang, chat_title: &str) -> String {
    match lang {
        Lang::En => format!("Deleting in «{chat_title}»"),
        Lang::Ru => format!("Удаление в «{chat_title}»"),
    }
}

pub fn format_delete_scanned(lang: Lang, n: u64) -> String {
    match lang {
        Lang::En => format!("Scanned: {n}"),
        Lang::Ru => format!("Просмотрено: {n}"),
    }
}

pub fn format_delete_deleted(lang: Lang, n: u64) -> String {
    match lang {
        Lang::En => format!("Deleted: {n}"),
        Lang::Ru => format!("Удалено: {n}"),
    }
}

pub fn format_delete_flood_wait(lang: Lang, secs: u32) -> String {
    match lang {
        Lang::En => format!("Telegram limit — waiting {secs} s…"),
        Lang::Ru => format!("Лимит Telegram — пауза {secs} с…"),
    }
}

fn format_delete_finished(
    lang: Lang,
    chat_title: &str,
    scanned: u64,
    deleted: u64,
    failed_batches: u32,
) -> String {
    match lang {
        Lang::En => {
            let mut s = format!(
                "Done «{chat_title}»: scanned {scanned}, deleted {deleted}."
            );
            if failed_batches > 0 {
                s.push_str(&format!(" Failed batches: {failed_batches}."));
            }
            s
        }
        Lang::Ru => {
            let mut s = format!(
                "Готово «{chat_title}»: просмотрено {scanned}, удалено {deleted}."
            );
            if failed_batches > 0 {
                s.push_str(&format!(" Неудачных пакетов: {failed_batches}."));
            }
            s
        }
    }
}

pub fn format_common_chats_section(lang: Lang, count: usize) -> String {
    match lang {
        Lang::En => format!("Common chats ({count})"),
        Lang::Ru => format!("Общие чаты ({count})"),
    }
}

pub fn chat_kind_label(lang: Lang, kind: &str) -> &'static str {
    match (lang, kind) {
        (Lang::En, "channel") => "channel",
        (Lang::Ru, "channel") => "канал",
        (Lang::En, "supergroup") => "super",
        (Lang::Ru, "supergroup") => "супер",
        (Lang::En, "group") => "group",
        (Lang::Ru, "group") => "группа",
        (Lang::En, _) => "chat",
        (Lang::Ru, _) => "чат",
    }
}

pub fn format_user_badges(lang: Lang, p: &UserProfile) -> String {
    let mut tags = Vec::new();
    if p.verified {
        tags.push(match lang {
            Lang::En => "verified",
            Lang::Ru => "верифицирован",
        });
    }
    if p.premium {
        tags.push("Premium");
    }
    if p.is_bot {
        tags.push(match lang {
            Lang::En => "bot",
            Lang::Ru => "бот",
        });
    }
    if p.contact {
        tags.push(match lang {
            Lang::En => "contact",
            Lang::Ru => "контакт",
        });
    }
    if p.mutual_contact {
        tags.push(match lang {
            Lang::En => "mutual",
            Lang::Ru => "взаимный",
        });
    }
    if p.scam {
        tags.push("scam");
    }
    if p.fake {
        tags.push("fake");
    }
    if tags.is_empty() {
        return match lang {
            Lang::En => "—".into(),
            Lang::Ru => "—".into(),
        };
    }
    tags.join(" · ")
}

fn format_common_chats(lang: Lang, chats: &[CommonChatSummary]) -> String {
    if chats.is_empty() {
        return match lang {
            Lang::En => "Common chats: —".into(),
            Lang::Ru => "Общие чаты: —".into(),
        };
    }
    let header = match lang {
        Lang::En => format!("Common chats ({}):", chats.len()),
        Lang::Ru => format!("Общие чаты ({}):", chats.len()),
    };
    let mut lines = vec![header];
    for c in chats {
        let line = match c.link.as_deref() {
            Some(url) => format!("  • {}  {url}", c.title),
            None => match lang {
                Lang::En => format!("  • {}  (no public link)", c.title),
                Lang::Ru => format!("  • {}  (нет публичной ссылки)", c.title),
            },
        };
        lines.push(line);
    }
    lines.join("\n")
}

fn format_user_profile(lang: Lang, p: &UserProfile) -> String {
    let username = p
        .username
        .as_ref()
        .map(|u| format!("@{u}"))
        .unwrap_or_else(|| dash(lang).into());
    let bio = if p.bio.is_empty() {
        dash(lang).into()
    } else {
        p.bio.clone()
    };
    let usernames = if p.usernames.is_empty() {
        dash(lang).into()
    } else {
        p.usernames
            .iter()
            .map(|u| format!("@{u}"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let json_line = p.json_path.as_ref().map(|path| match lang {
        Lang::En => format!("\nJSON: {}", path.display()),
        Lang::Ru => format!("\nJSON: {}", path.display()),
    }).unwrap_or_default();
    let common_block = format_common_chats(lang, &p.common_chats);

    match lang {
        Lang::En => format!(
            "Type: user\nID: {}\nName: {}\nFirst name: {}\nLast name: {}\nUsername: {username}\nUsernames: {usernames}\nPhone: {}\nBio: {bio}\nStatus: {}\nLang: {}\nBot: {}\nVerified: {}\nPremium: {}\nScam: {}\nFake: {}\nDeleted: {}\nRestricted: {}\nSupport: {}\nContact: {}\nMutual contact: {}\nSelf: {}\nRestrictions: {}\nCommon chats (count): {}\n{common_block}\nPhone calls: {}\nVideo calls: {}\n{json_line}\n{}",
            p.id,
            p.name,
            p.first_name,
            fmt_opt(lang, &p.last_name),
            fmt_opt(lang, &p.phone),
            p.status,
            fmt_opt(lang, &p.lang_code),
            fmt_bool(lang, p.is_bot),
            fmt_bool(lang, p.verified),
            fmt_bool(lang, p.premium),
            fmt_bool(lang, p.scam),
            fmt_bool(lang, p.fake),
            fmt_bool(lang, p.deleted),
            fmt_bool(lang, p.restricted),
            fmt_bool(lang, p.support),
            fmt_bool(lang, p.contact),
            fmt_bool(lang, p.mutual_contact),
            fmt_bool(lang, p.is_self),
            fmt_list(lang, &p.restriction_reasons),
            fmt_opt_i32(lang, p.common_chats_count),
            fmt_opt_bool(lang, p.phone_calls_available),
            fmt_opt_bool(lang, p.video_calls_available),
            format_photo(lang, &p.photo)
        ),
        Lang::Ru => format!(
            "Тип: пользователь\nID: {}\nИмя: {}\nИмя (first): {}\nФамилия: {}\nUsername: {username}\nUsernames: {usernames}\nТелефон: {}\nBio: {bio}\nСтатус: {}\nЯзык: {}\nБот: {}\nВерифицирован: {}\nPremium: {}\nScam: {}\nFake: {}\nУдалён: {}\nОграничен: {}\nSupport: {}\nКонтакт: {}\nВзаимный контакт: {}\nЭто я: {}\nОграничения: {}\nОбщие чаты (число): {}\n{common_block}\nЗвонки: {}\nВидеозвонки: {}\n{json_line}\n{}",
            p.id,
            p.name,
            p.first_name,
            fmt_opt(lang, &p.last_name),
            fmt_opt(lang, &p.phone),
            p.status,
            fmt_opt(lang, &p.lang_code),
            fmt_bool(lang, p.is_bot),
            fmt_bool(lang, p.verified),
            fmt_bool(lang, p.premium),
            fmt_bool(lang, p.scam),
            fmt_bool(lang, p.fake),
            fmt_bool(lang, p.deleted),
            fmt_bool(lang, p.restricted),
            fmt_bool(lang, p.support),
            fmt_bool(lang, p.contact),
            fmt_bool(lang, p.mutual_contact),
            fmt_bool(lang, p.is_self),
            fmt_list(lang, &p.restriction_reasons),
            fmt_opt_i32(lang, p.common_chats_count),
            fmt_opt_bool(lang, p.phone_calls_available),
            fmt_opt_bool(lang, p.video_calls_available),
            format_photo(lang, &p.photo)
        ),
    }
}

fn format_group_profile(lang: Lang, g: &GroupProfile) -> String {
    let kind = match (lang, g.is_megagroup) {
        (Lang::En, true) => "supergroup",
        (Lang::En, false) => "group",
        (Lang::Ru, true) => "супергруппа",
        (Lang::Ru, false) => "группа",
    };
    let username = g
        .username
        .as_ref()
        .map(|u| format!("@{u}"))
        .unwrap_or_else(|| dash(lang).into());
    let about = if g.about.is_empty() {
        dash(lang).into()
    } else {
        g.about.clone()
    };
    let json_line = g.json_path.as_ref().map(|path| match lang {
        Lang::En => format!("\nJSON: {}", path.display()),
        Lang::Ru => format!("\nJSON: {}", path.display()),
    }).unwrap_or_default();

    match lang {
        Lang::En => format!(
            "Type: {kind}\nID: {}\nTitle: {}\nUsername: {username}\nMembers: {}\nAbout: {about}{json_line}",
            g.id,
            g.title,
            fmt_opt_i32(lang, g.participants_count),
        ),
        Lang::Ru => format!(
            "Тип: {kind}\nID: {}\nНазвание: {}\nUsername: {username}\nУчастников: {}\nОписание: {about}{json_line}",
            g.id,
            g.title,
            fmt_opt_i32(lang, g.participants_count),
        ),
    }
}

fn format_channel_profile(lang: Lang, c: &ChannelProfile) -> String {
    let kind = match (lang, c.broadcast, c.megagroup) {
        (Lang::En, true, _) => "channel",
        (Lang::En, false, true) => "supergroup",
        (Lang::En, false, false) => "chat",
        (Lang::Ru, true, _) => "канал",
        (Lang::Ru, false, true) => "супергруппа",
        (Lang::Ru, false, false) => "чат",
    };
    let username = c
        .username
        .as_ref()
        .map(|u| format!("@{u}"))
        .unwrap_or_else(|| dash(lang).into());
    let usernames = if c.usernames.is_empty() {
        dash(lang).into()
    } else {
        c.usernames
            .iter()
            .map(|u| format!("@{u}"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let about = if c.about.is_empty() {
        dash(lang).into()
    } else {
        c.about.clone()
    };
    let json_line = c.json_path.as_ref().map(|path| match lang {
        Lang::En => format!("\nJSON: {}", path.display()),
        Lang::Ru => format!("\nJSON: {}", path.display()),
    }).unwrap_or_default();

    match lang {
        Lang::En => format!(
            "Type: {kind}\nID: {}\nTitle: {}\nUsername: {username}\nUsernames: {usernames}\nAbout: {about}\nSubscribers: {}\nVerified: {}\nScam: {}\nFake: {}\nRestricted: {}\nRestrictions: {}\n{json_line}\n{}",
            c.id,
            c.title,
            fmt_opt_i32(lang, c.participants_count),
            fmt_bool(lang, c.verified),
            fmt_bool(lang, c.scam),
            fmt_bool(lang, c.fake),
            fmt_bool(lang, c.restricted),
            fmt_list(lang, &c.restriction_reasons),
            format_photo(lang, &c.photo)
        ),
        Lang::Ru => format!(
            "Тип: {kind}\nID: {}\nНазвание: {}\nUsername: {username}\nUsernames: {usernames}\nОписание: {about}\nПодписчиков: {}\nВерифицирован: {}\nScam: {}\nFake: {}\nОграничен: {}\nОграничения: {}\n{json_line}\n{}",
            c.id,
            c.title,
            fmt_opt_i32(lang, c.participants_count),
            fmt_bool(lang, c.verified),
            fmt_bool(lang, c.scam),
            fmt_bool(lang, c.fake),
            fmt_bool(lang, c.restricted),
            fmt_list(lang, &c.restriction_reasons),
            format_photo(lang, &c.photo)
        ),
    }
}

pub fn workdir_line(lang: Lang, path: &str) -> String {
    format!("{} {}", t(lang, I18nKey::Workdir), path)
}
