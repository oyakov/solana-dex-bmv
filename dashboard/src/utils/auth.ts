export const getAuthToken = () => {
    if (typeof window === "undefined") return null;
    return localStorage.getItem("bmv_auth_token");
};

export const getAuthHeaders = () => {
    const token = getAuthToken();
    return {
        "Content-Type": "application/json",
        ...(token ? { Authorization: `Bearer ${token}` } : {}),
    };
};

export const logout = () => {
    localStorage.removeItem("bmv_auth_token");
    document.cookie = "bmv_auth_token=; path=/; expires=Thu, 01 Jan 1970 00:00:01 GMT;";
    window.location.href = "/login";
};
