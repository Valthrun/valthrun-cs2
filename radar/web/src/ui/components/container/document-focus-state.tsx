import * as React from "react";
import { useContext, useEffect, useState } from "react";

const ContextDocumentFocusState = React.createContext<boolean>(false);
export const DocumentFocusStateProvider = (props: {
    children: React.ReactNode
}) => {
    const [focus, updateFocus] = useState<boolean>(document.hasFocus());
    useEffect(() => {
        const setFocus = () => updateFocus(true);
        const clearFocus = () => updateFocus(false);

        // Moving focus between elements does not mean the page lost focus.
        window.addEventListener("focus", setFocus);
        window.addEventListener("blur", clearFocus);

        document.addEventListener("mouseenter", setFocus);
        document.addEventListener("mouseleave", clearFocus);
        return () => {
            window.removeEventListener("focus", setFocus);
            window.removeEventListener("blur", clearFocus);

            document.removeEventListener("mouseenter", setFocus);
            document.removeEventListener("mouseleave", clearFocus);
        }
    }, []);

    return (
        <ContextDocumentFocusState.Provider value={focus}>
            {props.children}
        </ContextDocumentFocusState.Provider>
    )
};

export const useDocumentFocusState = () => useContext(ContextDocumentFocusState);