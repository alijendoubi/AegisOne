import Header from "@/components/Header";
import Footer from "@/components/Footer";
import Hero from "@/components/Hero";
import TrustedBy from "@/components/TrustedBy";
import Features from "@/components/Features";
import HowItWorks from "@/components/HowItWorks";
import Demo from "@/components/Demo";
import Pricing from "@/components/Pricing";
import FAQ from "@/components/FAQ";
import WaitlistSection from "@/components/WaitlistSection";
import RevealObserver from "@/components/RevealObserver";

export default function Home() {
  return (
    <>
      <Header />
      <main>
        <Hero />
        <TrustedBy />
        <Features />
        <HowItWorks />
        <Demo />
        <Pricing />
        <FAQ />
        <WaitlistSection />
      </main>
      <Footer />
      {/* Client-side reveal-on-scroll observer */}
      <RevealObserver />
    </>
  );
}
